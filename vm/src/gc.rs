//! Mark-sweep collection over the heap's intrusive object list.
//!
//! The VM gathers roots and drives collection at safe points (see
//! `VirtualMachine::collect_garbage`); this module owns the *tracing* — which
//! heap edges each object kind has. Sweeping lives on `Storage`, which owns the
//! allocation list. Interned strings (`Value::Symbol`) are permanent and never
//! traced or swept.

use intrusive_collections::UnsafeRef;

use crate::object::{
    ObjKind, Object, bound_method::LoxBoundMethod, class::LoxClass, closure::LoxClosure,
    function::LoxFunction, instance::LoxInstance, upvalue::LoxUpvalue,
};
use crate::value::Value;

/// Worklist of marked-but-not-yet-traced ("gray") objects. The VM marks roots
/// into it, then [`trace`](Self::trace) blackens the whole reachable graph.
#[derive(Default)]
pub struct Tracer {
    gray: Vec<UnsafeRef<Object>>,
}

impl Tracer {
    pub fn mark_value(&mut self, value: &Value) {
        if let Value::Object(obj) = value {
            self.mark_obj(obj);
        }
    }

    pub fn mark_obj(&mut self, obj: &UnsafeRef<Object>) {
        if !obj.is_marked() {
            obj.set_marked(true);
            self.gray.push(obj.clone());
        }
    }

    /// Blacken every reachable object: trace each gray object's edges until the
    /// worklist drains.
    pub fn trace(&mut self) {
        while let Some(obj) = self.gray.pop() {
            blacken(&obj, self);
        }
    }
}

fn blacken(obj: &UnsafeRef<Object>, tracer: &mut Tracer) {
    // SAFETY in every arm: the matched `ObjKind` witnesses the dynamic type.
    match obj.kind() {
        // Leaves: a string owns only bytes; a native owns only a fn pointer.
        ObjKind::String | ObjKind::Native => {}
        ObjKind::Function => {
            let function = unsafe { obj.downcast_ref::<LoxFunction>() };
            for constant in &function.chunk().constants {
                tracer.mark_value(constant);
            }
        }
        ObjKind::Closure => {
            let closure = unsafe { obj.downcast_ref::<LoxClosure>() };
            tracer.mark_obj(closure.function_handle());
            for upvalue in closure.upvalues() {
                tracer.mark_obj(upvalue);
            }
        }
        ObjKind::Upvalue => {
            let upvalue = unsafe { obj.downcast_ref::<LoxUpvalue>() };
            // An open upvalue's value is on the stack (already a root); only a
            // closed upvalue owns it.
            if let Some(value) = upvalue.closed_value() {
                tracer.mark_value(&value);
            }
        }
        ObjKind::Class => {
            let class = unsafe { obj.downcast_ref::<LoxClass>() };
            class.trace_methods(|value| tracer.mark_value(value));
        }
        ObjKind::Instance => {
            let instance = unsafe { obj.downcast_ref::<LoxInstance>() };
            tracer.mark_obj(instance.class_handle());
            instance.trace_fields(|value| tracer.mark_value(value));
        }
        ObjKind::BoundMethod => {
            let bound = unsafe { obj.downcast_ref::<LoxBoundMethod>() };
            tracer.mark_value(bound.receiver());
            tracer.mark_obj(bound.method());
        }
    }
}
