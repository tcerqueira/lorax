use std::{
    fmt::Display,
    mem,
    ops::{Add, Div, Mul, Sub},
};

use intrusive_collections::UnsafeRef;
use lasso::Spur;
use report::{Span, error::RuntimeError};
use smallvec::SmallVec;

use crate::{
    chunk::Chunk,
    debug::LineInfo,
    enconding::OpCode,
    gc::Tracer,
    object::{
        ObjKind, Object,
        bound_method::LoxBoundMethod,
        class::LoxClass,
        closure::LoxClosure,
        instance::LoxInstance,
        native::{LoxNative, NativeFn},
        string::LoxString,
        upvalue::LoxUpvalue,
    },
    storage::{Storage, SymbolMap, WithStorage},
    value::{Addr, Value, ValueError},
    vm::{
        error::VirtualMachineError,
        frame::{CallFrame, FrameSource},
        stack::Stack,
    },
};

pub mod error;
pub mod frame;
pub mod stack;

/// Maximum call depth before a graceful `Stack overflow.` runtime error (clox's
/// `FRAMES_MAX`). Keeps Lox recursion from riding the Rust call stack into an
/// uncatchable abort.
const FRAMES_MAX: usize = 64;

pub struct VirtualMachine {
    stack: Stack,
    frames: Vec<CallFrame>,
    storage: Storage,
    globals: SymbolMap<Value>,
    /// Live `LoxUpvalue`s still pointing into the value stack, so sibling
    /// closures capturing the same local share one upvalue. Closed and removed
    /// when the underlying slot leaves the stack. Usually only a few are open at
    /// once, so it lives inline (no heap) until that handful is exceeded.
    open_upvalues: SmallVec<[UnsafeRef<Object>; 8]>,
    /// Interned name of the initializer method (`init`), cached for the
    /// class-construction fast path.
    init: Spur,
    /// Collect on every safe point regardless of the threshold — exercises the
    /// otherwise-untriggered collector (tests, Miri, fuzzing).
    stress_gc: bool,
    debug: bool,
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
    pub fn new() -> Self {
        let mut storage = Storage::new();
        let init = storage.intern("init");
        let mut vm = Self {
            stack: Stack::default(),
            frames: Vec::with_capacity(FRAMES_MAX),
            storage,
            globals: SymbolMap::default(),
            open_upvalues: SmallVec::new(),
            init,
            stress_gc: std::env::var_os("LORAX_STRESS_GC").is_some(),
            debug: false,
        };
        vm.define_native("clock", natives::clock);
        vm
    }

    pub fn debug() -> Self {
        Self {
            debug: true,
            ..Self::new()
        }
    }

    /// Like [`new`](Self::new) but collects garbage at every safe point.
    pub fn stress() -> Self {
        Self {
            stress_gc: true,
            ..Self::new()
        }
    }

    pub fn storage(&mut self) -> &mut Storage {
        &mut self.storage
    }

    fn define_native(&mut self, name: &str, func: NativeFn) {
        let spur = self.storage.intern(name);
        let obj = self.storage.add_obj(Box::new(LoxNative::new(spur, func)));
        self.globals.insert(spur, Value::object(obj));
    }

    pub fn run(&mut self, chunk: Chunk) -> Result<(), VirtualMachineError> {
        // Reset transient execution state so a previous run that errored
        // mid-expression (e.g. a prior REPL line) can't leak stack slots or open
        // upvalues into this one. Globals live separately and persist.
        self.stack.clear();
        self.open_upvalues.clear();
        self.frames.clear();
        self.push_frame(FrameSource::TopLevel(Box::new(chunk)), 0);
        self.dispatch()
    }

    fn dispatch(&mut self) -> Result<(), VirtualMachineError> {
        loop {
            // Safe point: between instructions every live object is reachable
            // from a root (stack, globals, frames, open upvalues).
            self.maybe_collect();

            let top = self.frames.len() - 1;
            // Decode the next instruction off the current frame's code, then
            // release the code borrow (the op is `Copy`) before any arm touches
            // `&mut self`. `op_start` is the byte offset of this instruction,
            // used for error spans.
            let op_start = self.frames[top].ip;
            let op = {
                let frame = &mut self.frames[top];
                let mut ip = frame.ip;
                let op = OpCode::decode_at(frame.code(), &mut ip)?;
                frame.ip = ip;
                op
            };
            self.trace(op);
            let base = self.frames[top].base;

            match op {
                OpCode::NoOp => {}
                OpCode::Ret => {
                    let result = self.stack.pop();
                    let frame = self.frames.pop().expect("active frame on return");
                    // Close everything the frame owned before its window is gone.
                    self.close_upvalues(frame.base);
                    self.stack.truncate(frame.base);
                    if self.frames.is_empty() {
                        return Ok(());
                    }
                    self.stack.push(result);
                }
                OpCode::Call(arg_count) => self.call_value(arg_count, op_start)?,
                OpCode::Closure(addr) => self.make_closure(addr, base),
                OpCode::GetUpvalue(slot) => {
                    let value = self.upvalue_get(slot);
                    self.stack.push(value);
                }
                OpCode::SetUpvalue(slot) => {
                    // Assignment is an expression — leave the value on the stack.
                    let value = *self.stack.top();
                    self.upvalue_set(slot, value);
                }
                OpCode::CloseUpvalue => {
                    self.close_upvalues(self.stack.len() - 1);
                    self.stack.pop();
                }
                OpCode::Class(addr) => {
                    let name = self.variable_name(addr);
                    let class = self.storage.add_obj(Box::new(LoxClass::new(name)));
                    self.stack.push(Value::object(class));
                }
                OpCode::GetProperty(addr) => self.get_property(addr, op_start)?,
                OpCode::SetProperty(addr) => self.set_property(addr, op_start)?,
                OpCode::Method(addr) => self.define_method(addr),
                OpCode::Invoke(addr, arg_count) => self.invoke(addr, arg_count, op_start)?,
                OpCode::Inherit => self.inherit(op_start)?,
                OpCode::GetSuper(addr) => self.get_super(addr, op_start)?,
                OpCode::SuperInvoke(addr, arg_count) => {
                    self.super_invoke(addr, arg_count, op_start)?
                }
                OpCode::Constant(addr) => {
                    let constant = *self.current_chunk().constant(addr);
                    self.stack.push(constant);
                }
                OpCode::Neg => {
                    // Negate the top slot in place (like `Not`): no pop/push.
                    let v = self.stack.top_mut();
                    match -*v {
                        Ok(r) => *v = r,
                        Err(_) => return Err(self.runtime(op_start, "invalid operand")),
                    }
                }
                OpCode::Add if self.stack.peek(0).is_str() && self.stack.peek(1).is_str() => {
                    self.concatenate_str()
                }
                OpCode::Add => self.binary(Value::add, op_start)?,
                OpCode::Sub => self.binary(Value::sub, op_start)?,
                OpCode::Mul => self.binary(Value::mul, op_start)?,
                OpCode::Div => self.binary(Value::div, op_start)?,
                OpCode::True => self.stack.push(Value::boolean(true)),
                OpCode::False => self.stack.push(Value::boolean(false)),
                OpCode::Nil => self.stack.push(Value::nil()),
                OpCode::Not => {
                    let v = self.stack.top_mut();
                    *v = Value::boolean(v.is_falsey());
                }
                OpCode::Equal => self.equal(),
                OpCode::Greater => self.binary(Value::greater, op_start)?,
                OpCode::Less => self.binary(Value::less, op_start)?,
                OpCode::Print => {
                    let v = self.stack.pop();
                    println!("{}", WithStorage(&v, &self.storage));
                }
                OpCode::Pop => _ = self.stack.pop(),
                OpCode::PopN(n) => self.stack.pop_n(n),
                OpCode::DefGlobal(addr) => {
                    self.with_variable(addr, |vm, key, value| {
                        vm.globals.insert(key, value);
                    });
                    self.stack.pop();
                }
                OpCode::GetGlobal(addr) => {
                    let key = self.variable_name(addr);
                    match self.globals.get(&key) {
                        Some(value) => {
                            let value = *value;
                            self.stack.push(value);
                        }
                        None => return Err(self.runtime(op_start, "Undefined variable.")),
                    }
                }
                OpCode::SetGlobal(addr) => {
                    let key = self.variable_name(addr);
                    let value = *self.stack.top();
                    if let Some(slot) = self.globals.get_mut(&key) {
                        *slot = value;
                    } else {
                        return Err(self.runtime(op_start, "Undefined variable."));
                    }
                }
                OpCode::GetLocal(slot) => {
                    let v = *self.stack.at(base + slot as usize);
                    self.stack.push(v);
                }
                OpCode::SetLocal(slot) => {
                    // Assignment is an expression — leave the value on top so
                    // chained uses like `print a = 1;` work.
                    let v = *self.stack.top();
                    *self.stack.at_mut(base + slot as usize) = v;
                }
                OpCode::JmpIfFalse(offset) => {
                    if self.stack.top().is_falsey() {
                        self.frames[top].ip += offset as usize;
                    }
                }
                OpCode::JmpIfFalsePop(offset) => {
                    // Pop the condition unconditionally, jump if it was falsey.
                    if self.stack.pop().is_falsey() {
                        self.frames[top].ip += offset as usize;
                    }
                }
                OpCode::JmpIfTrue(offset) => {
                    if !self.stack.top().is_falsey() {
                        self.frames[top].ip += offset as usize;
                    }
                }
                OpCode::Jmp(offset) => self.frames[top].ip += offset as usize,
                OpCode::Loop(offset) => self.frames[top].ip -= offset as usize,
            }
        }
    }

    fn current_chunk(&self) -> &Chunk {
        // Cached chunk pointer (no closure→function→chunk downcast).
        self.frames.last().expect("no active frame").chunk()
    }

    /// Push a frame and populate its cached chunk pointer from its final `Vec`
    /// slot. The `frames` `Vec` is pre-sized to `FRAMES_MAX` and depth is capped
    /// before any closure call, so it never reallocates mid-run.
    fn push_frame(&mut self, source: FrameSource, base: usize) {
        self.frames.push(CallFrame::new(source, base));
        self.frames
            .last_mut()
            .expect("frame just pushed")
            .cache_chunk();
    }

    fn span_at(&self, ip: usize) -> Span {
        self.frames
            .last()
            .and_then(|f| f.source.chunk().get_line(ip as u64))
            .map(LineInfo::to_span)
            .unwrap_or_default()
    }

    fn runtime(&self, ip: usize, message: impl Display) -> VirtualMachineError {
        RuntimeError::custom(self.span_at(ip), message).into()
    }

    fn maybe_collect(&mut self) {
        if self.stress_gc || self.storage.should_collect() {
            self.collect_garbage();
        }
    }

    /// Mark every object reachable from a root, then sweep the rest. Roots are
    /// the value stack, globals, each frame's closure, the open upvalues, and
    /// the script chunk's constants (which live outside the heap).
    fn collect_garbage(&mut self) {
        let mut tracer = Tracer::default();
        for value in self.stack.iter() {
            tracer.mark_value(value);
        }
        for value in self.globals.values() {
            tracer.mark_value(value);
        }
        for frame in &self.frames {
            match &frame.source {
                FrameSource::Closure(obj) => tracer.mark_obj(obj),
                FrameSource::TopLevel(chunk) => {
                    for constant in &chunk.constants {
                        tracer.mark_value(constant);
                    }
                }
            }
        }
        for upvalue in &self.open_upvalues {
            tracer.mark_obj(upvalue);
        }
        tracer.trace();
        self.storage.sweep();
    }

    /// Invoke the value `arg_count` slots below the top: push a frame for a
    /// function, or run a native inline. The callee sits at `peek(arg_count)`
    /// with its arguments stacked above it (the overlapping-window convention).
    fn call_value(&mut self, arg_count: u8, ip: usize) -> Result<(), VirtualMachineError> {
        if let Some(obj) = self.stack.peek(arg_count as usize).as_object() {
            match obj.kind() {
                ObjKind::Closure => return self.call_closure(obj, arg_count, ip),
                ObjKind::Native => return self.call_native(obj, arg_count),
                ObjKind::Class => return self.call_class(obj, arg_count, ip),
                ObjKind::BoundMethod => return self.call_bound_method(obj, arg_count, ip),
                _ => {}
            }
        }
        Err(self.runtime(ip, "Can only call functions and classes."))
    }

    /// Calling a class constructs an instance: the new instance replaces the
    /// class in the callee slot, then its `init` (if any) runs as a method over
    /// it. With no `init`, the call takes no arguments and yields the instance.
    fn call_class(
        &mut self,
        class: UnsafeRef<Object>,
        arg_count: u8,
        ip: usize,
    ) -> Result<(), VirtualMachineError> {
        let instance = self
            .storage
            .add_obj(Box::new(LoxInstance::new(class.clone())));
        *self.stack.peek_mut(arg_count as usize) = Value::object(instance);
        // SAFETY: dispatched on ObjKind::Class.
        match unsafe { class.downcast_ref::<LoxClass>() }.method(self.init) {
            Some(init) => {
                let closure = init.as_object().expect("method table holds closures");
                self.call_closure(closure, arg_count, ip)
            }
            None if arg_count != 0 => {
                Err(RuntimeError::arity(self.span_at(ip), 0, arg_count as usize).into())
            }
            None => Ok(()),
        }
    }

    fn call_closure(
        &mut self,
        obj: UnsafeRef<Object>,
        arg_count: u8,
        ip: usize,
    ) -> Result<(), VirtualMachineError> {
        // SAFETY: dispatched on ObjKind::Closure.
        let arity = unsafe { obj.downcast_ref::<LoxClosure>() }.arity();
        if arg_count != arity {
            return Err(RuntimeError::arity(self.span_at(ip), arity, arg_count as usize).into());
        }
        if self.frames.len() >= FRAMES_MAX {
            return Err(self.runtime(ip, "Stack overflow."));
        }
        let base = self.stack.len() - arg_count as usize - 1;
        self.push_frame(FrameSource::Closure(obj), base);
        Ok(())
    }

    fn get_property(&mut self, addr: Addr, ip: usize) -> Result<(), VirtualMachineError> {
        if !self.stack.peek(0).is_instance() {
            return Err(self.runtime(ip, "Only instances have properties."));
        }
        let name = self.variable_name(addr);
        // Keep the instance on the stack as a root while reading it.
        let instance = *self.stack.peek(0);

        // A field shadows a method.
        if let Some(value) = self.as_instance(&instance).field(name) {
            self.stack.pop();
            self.stack.push(value);
            return Ok(());
        }
        if let Some(method) = self.as_instance(&instance).find_method(name) {
            let bound = self.bind_method(instance, method);
            self.stack.pop();
            self.stack.push(bound);
            return Ok(());
        }
        Err(self.undefined_property(name, ip))
    }

    /// Downcast a value the caller has already confirmed is an instance.
    fn as_instance<'a>(&self, value: &'a Value) -> &'a LoxInstance {
        let Some(obj) = value.as_object() else {
            unreachable!("caller checked is_instance")
        };
        let instance = unsafe { obj.downcast_ref::<LoxInstance>() };
        // SAFETY: the caller verified the object is an instance. `obj` is a fresh
        // non-owning handle reconstructed from `value`'s bits, so the borrow is
        // tied to that temporary; widening it to `'a` is sound *structurally*
        // (a `u64` `Value` roots nothing): the GC runs only at the dispatch safe
        // point, `add_obj` never collects, and no opcode handler reaches a safe
        // point while a borrow obtained here is live — so the instance cannot be
        // freed before `'a` ends, and the collector never moves it.
        unsafe { mem::transmute::<&LoxInstance, &'a LoxInstance>(instance) }
    }

    fn undefined_property(&self, name: Spur, ip: usize) -> VirtualMachineError {
        let name = self.storage.resolve(name);
        self.runtime(ip, format!("Undefined property '{name}'."))
    }

    fn bind_method(&mut self, receiver: Value, method: Value) -> Value {
        let Some(closure) = method.as_object() else {
            panic!("method table holds a non-closure")
        };
        let bound = self
            .storage
            .add_obj(Box::new(LoxBoundMethod::new(receiver, closure)));
        Value::object(bound)
    }

    fn define_method(&mut self, addr: Addr) {
        let name = self.variable_name(addr);
        let method = self.stack.pop();
        let Some(class) = self.stack.peek(0).as_object() else {
            panic!("OP_METHOD expects a class beneath the method")
        };
        // SAFETY: the compiler emits OP_METHOD only with a class on the stack.
        unsafe { class.downcast_ref::<LoxClass>() }.define_method(name, method);
    }

    fn invoke(&mut self, addr: Addr, arg_count: u8, ip: usize) -> Result<(), VirtualMachineError> {
        if !self.stack.peek(arg_count as usize).is_instance() {
            return Err(self.runtime(ip, "Only instances have methods."));
        }
        let name = self.variable_name(addr);
        let receiver = *self.stack.peek(arg_count as usize);

        // A field shadowing a method is called as an ordinary value.
        if let Some(field) = self.as_instance(&receiver).field(name) {
            *self.stack.peek_mut(arg_count as usize) = field;
            return self.call_value(arg_count, ip);
        }
        match self.as_instance(&receiver).find_method(name) {
            Some(method) => {
                let closure = method.as_object().expect("method table holds closures");
                self.call_closure(closure, arg_count, ip)
            }
            None => Err(self.undefined_property(name, ip)),
        }
    }

    fn inherit(&mut self, ip: usize) -> Result<(), VirtualMachineError> {
        // Stack: [.., superclass, subclass].
        let superclass = *self.stack.peek(1);
        let Some(super_obj) = superclass.as_object() else {
            return Err(self.runtime(ip, "Superclass must be a class."));
        };
        if super_obj.kind() != ObjKind::Class {
            return Err(self.runtime(ip, "Superclass must be a class."));
        }
        let subclass = *self.stack.peek(0);
        let Some(sub_obj) = subclass.as_object() else {
            unreachable!("the compiler always emits a class as the subclass")
        };
        // SAFETY: super_obj is a class (checked); sub_obj is a class (compiler).
        let super_class = unsafe { super_obj.downcast_ref::<LoxClass>() };
        unsafe { sub_obj.downcast_ref::<LoxClass>() }.copy_methods_from(super_class);
        self.stack.pop(); // subclass; the superclass stays as the `super` local
        Ok(())
    }

    /// Pop the superclass (top of stack) and look up `addr`'s method on it.
    fn pop_super_method(&mut self, addr: Addr, ip: usize) -> Result<Value, VirtualMachineError> {
        let name = self.variable_name(addr);
        let superclass = self.stack.pop();
        let Some(super_obj) = superclass.as_object() else {
            unreachable!("the compiler always loads a class for super")
        };
        // SAFETY: a `super` local always holds a class.
        unsafe { super_obj.downcast_ref::<LoxClass>() }
            .method(name)
            .ok_or_else(|| self.undefined_property(name, ip))
    }

    fn get_super(&mut self, addr: Addr, ip: usize) -> Result<(), VirtualMachineError> {
        // Stack: [.., receiver, superclass]; pop the class, bind to the receiver.
        let method = self.pop_super_method(addr, ip)?;
        let receiver = *self.stack.peek(0);
        let bound = self.bind_method(receiver, method);
        self.stack.pop(); // receiver
        self.stack.push(bound);
        Ok(())
    }

    fn super_invoke(
        &mut self,
        addr: Addr,
        arg_count: u8,
        ip: usize,
    ) -> Result<(), VirtualMachineError> {
        // Stack: [.., receiver, args.., superclass]; pop the class, call directly.
        let Some(closure) = self.pop_super_method(addr, ip)?.as_object() else {
            unreachable!("method table holds closures")
        };
        self.call_closure(closure, arg_count, ip)
    }

    fn call_bound_method(
        &mut self,
        obj: UnsafeRef<Object>,
        arg_count: u8,
        ip: usize,
    ) -> Result<(), VirtualMachineError> {
        // SAFETY: dispatched on ObjKind::BoundMethod.
        let bound = unsafe { obj.downcast_ref::<LoxBoundMethod>() };
        let receiver = *bound.receiver();
        let method = bound.method().clone();
        // The receiver becomes slot 0 (`this`) of the method's frame.
        *self.stack.peek_mut(arg_count as usize) = receiver;
        self.call_closure(method, arg_count, ip)
    }

    fn set_property(&mut self, addr: Addr, ip: usize) -> Result<(), VirtualMachineError> {
        if !self.stack.peek(1).is_instance() {
            return Err(self.runtime(ip, "Only instances have fields."));
        }
        let name = self.variable_name(addr);
        let value = *self.stack.peek(0);
        let instance = *self.stack.peek(1);
        self.as_instance(&instance).set_field(name, value);
        self.stack.pop(); // value
        self.stack.pop(); // instance
        self.stack.push(value);
        Ok(())
    }

    /// Build a closure from the function constant at `addr`, capturing each
    /// upvalue from its `(is_local, index)` tail: a local of the current frame
    /// (`base + index`) or an upvalue of the current closure.
    fn make_closure(&mut self, addr: Addr, base: usize) {
        let function = match self.current_chunk().constant(addr).as_object() {
            Some(obj) => obj,
            None => panic!("OP_CLOSURE constant is not an object"),
        };
        let count = self.current_chunk().closure_upvalue_count(addr);
        let mut upvalues: SmallVec<[UnsafeRef<Object>; 8]> = SmallVec::with_capacity(count);
        for _ in 0..count {
            let is_local = self.read_byte() != 0;
            let index = self.read_byte();
            let upvalue = if is_local {
                self.capture_upvalue(base + index as usize)
            } else {
                self.current_closure_upvalue(index)
            };
            upvalues.push(upvalue);
        }
        // `LoxClosure` stores its upvalues inline (a DST), so it is one
        // allocation. The captures gathered above are all reachable from a root
        // when `add_obj` may collect: a local capture is pushed onto
        // `open_upvalues` by `capture_upvalue`, and a non-local capture is an
        // upvalue of the current closure (rooted via this frame). The `function`
        // itself is a chunk constant. So copying them into the closure's tail is
        // GC-safe.
        let obj = self.storage.add_obj(LoxClosure::boxed(function, &upvalues));
        self.stack.push(Value::object(obj));
    }

    /// Read one operand byte from the current frame's code, advancing its ip.
    /// Used for the `OP_CLOSURE` upvalue tail.
    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("active frame");
        let byte = frame.code()[frame.ip];
        frame.ip += 1;
        byte
    }

    /// Find the open upvalue already capturing `stack_index` (so siblings share
    /// it), or allocate a fresh one and track it as open.
    fn capture_upvalue(&mut self, stack_index: usize) -> UnsafeRef<Object> {
        for open in &self.open_upvalues {
            // SAFETY: open_upvalues holds LoxUpvalue handles.
            if unsafe { open.downcast_ref::<LoxUpvalue>() }.open_index() == Some(stack_index) {
                return open.clone();
            }
        }
        let obj = self
            .storage
            .add_obj(Box::new(LoxUpvalue::open(stack_index)));
        self.open_upvalues.push(obj.clone());
        obj
    }

    /// Close (hoist off the stack into their own cell) every open upvalue at or
    /// above `from`, removing them from the open set.
    fn close_upvalues(&mut self, from: usize) {
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let handle = self.open_upvalues[i].clone();
            // SAFETY: open_upvalues holds LoxUpvalue handles.
            let upvalue = unsafe { handle.downcast_ref::<LoxUpvalue>() };
            match upvalue.open_index() {
                Some(index) if index >= from => {
                    let value = *self.stack.at(index);
                    upvalue.close(value);
                    self.open_upvalues.swap_remove(i);
                }
                _ => i += 1,
            }
        }
    }

    fn current_closure_upvalue(&self, index: u8) -> UnsafeRef<Object> {
        match &self.frames.last().expect("active frame").source {
            // SAFETY: a Closure frame holds a LoxClosure.
            FrameSource::Closure(obj) => unsafe { obj.downcast_ref::<LoxClosure>() }
                .upvalue(index)
                .clone(),
            FrameSource::TopLevel(_) => panic!("upvalue access in the top-level frame"),
        }
    }

    fn upvalue_get(&self, slot: u8) -> Value {
        let handle = self.current_closure_upvalue(slot);
        // SAFETY: a closure's upvalue array holds LoxUpvalue handles.
        let upvalue = unsafe { handle.downcast_ref::<LoxUpvalue>() };
        match upvalue.open_index() {
            Some(index) => *self.stack.at(index),
            None => upvalue
                .closed_value()
                .expect("closed upvalue holds a value"),
        }
    }

    fn upvalue_set(&mut self, slot: u8, value: Value) {
        let handle = self.current_closure_upvalue(slot);
        // SAFETY: a closure's upvalue array holds LoxUpvalue handles.
        let upvalue = unsafe { handle.downcast_ref::<LoxUpvalue>() };
        match upvalue.open_index() {
            Some(index) => *self.stack.at_mut(index) = value,
            None => upvalue.close(value),
        }
    }

    fn call_native(
        &mut self,
        obj: UnsafeRef<Object>,
        arg_count: u8,
    ) -> Result<(), VirtualMachineError> {
        // SAFETY: dispatched on ObjKind::Native.
        let func = unsafe { obj.downcast_ref::<LoxNative>() }.func();
        let arg_start = self.stack.len() - arg_count as usize;
        // The args stay on the stack (rooted) for the call; pass them as a slice
        // rather than cloning into a Vec. `storage` and `stack` are disjoint fields.
        let result = func(&mut self.storage, self.stack.args_from(arg_start))?;
        self.stack.truncate(arg_start - 1); // discard callee + arguments
        self.stack.push(result);
        Ok(())
    }

    fn binary(
        &mut self,
        op: fn(Value, Value) -> Result<Value, ValueError>,
        ip: usize,
    ) -> Result<(), VirtualMachineError> {
        // Pop only the right operand; apply the op to the left operand's slot in
        // place, so the common arithmetic op is one pop + one write, not two pops
        // and a push.
        let b = self.stack.pop();
        let a = self.stack.top_mut();
        match op(*a, b) {
            Ok(res) => {
                *a = res;
                Ok(())
            }
            Err(_) => Err(self.runtime(ip, "invalid operand")),
        }
    }

    fn equal(&mut self) {
        let b = self.stack.pop();
        let a = self.stack.pop();
        // `PartialEq` settles bit-identity in one comparison: same number (IEEE),
        // nil, bool, the same interned symbol, the same object. If that fails, two
        // *distinct* symbols still can't be equal — interning makes a distinct
        // `Spur` imply distinct text — so we skip the text compare. Only a string
        // pair that isn't two symbols (a `Symbol` vs a heap `LoxString`, or two
        // `LoxString`s) needs the text fallback that plain `==` misses.
        let res = if a == b {
            true
        } else if a.is_symbol() && b.is_symbol() {
            false
        } else {
            a.is_str() && b.is_str() && a.as_str(&self.storage) == b.as_str(&self.storage)
        };
        self.stack.push(Value::boolean(res));
    }

    fn concatenate_str(&mut self) {
        // Build the joined string while the operands stay on the stack as GC roots.
        let concatenated = {
            let a = self.stack.peek(1).as_str(&self.storage);
            let b = self.stack.peek(0).as_str(&self.storage);
            LoxString::concat(a, b)
        };
        let obj = self.storage.add_obj(concatenated);
        self.stack.pop();
        self.stack.pop();
        self.stack.push(Value::object(obj));
    }

    fn with_variable<T>(
        &mut self,
        addr: Addr,
        f: impl FnOnce(&mut VirtualMachine, Spur, Value) -> T,
    ) -> T {
        // Value stays on the stack across `f` so it remains a GC root if a
        // future collector triggers during a globals rehash.
        let key = self.variable_name(addr);
        let value = *self.stack.top();
        f(self, key, value)
    }

    fn variable_name(&self, addr: Addr) -> Spur {
        let Some(key) = self.current_chunk().constant(addr).as_symbol() else {
            panic!("could not get variable name: constant slot is not a Symbol")
        };
        key
    }

    fn trace(&self, op: OpCode) {
        if !self.debug {
            return;
        }
        println!("--> {:?}", op);
        print!("--> {:>16}", "stack: [ ");
        let mut stack_iter = self.stack.iter().peekable();
        while let Some(value) = stack_iter.next() {
            print!("{value}");
            if stack_iter.peek().is_some() {
                print!(", ");
            }
        }
        println!(" ]");
    }
}

mod natives {
    use std::{sync::OnceLock, time::Instant};

    use report::error::RuntimeError;

    use crate::{storage::Storage, value::Value};

    /// Seconds elapsed since the first `clock()` call. The baseline lives in a
    /// process-wide `OnceLock` because a bare `fn` native can't capture state;
    /// only elapsed deltas are meaningful, so a lazy baseline is fine.
    pub fn clock(_storage: &mut Storage, _args: &[Value]) -> Result<Value, RuntimeError> {
        static START: OnceLock<Instant> = OnceLock::new();
        let start = START.get_or_init(Instant::now);
        Ok(Value::number(start.elapsed().as_secs_f64()))
    }
}
