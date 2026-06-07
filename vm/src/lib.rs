#![feature(ptr_metadata)]
#![feature(arbitrary_self_types)]

use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use anyhow::Context;
use lexer::Scanner;
use report::{Error, Reporter};

use crate::{
    compiler::Compiler,
    vm::{VirtualMachine, error::VirtualMachineError},
};

pub mod chunk;
pub mod compiler;
pub(crate) mod debug;
pub mod enconding;
pub mod gc;
pub mod object;
pub mod storage;
pub mod value;
pub mod vm;

pub fn run_file(path: &Path) -> Result<(), Error> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read source file {}", path.display()))?;
    run(source, &mut VirtualMachine::default())
}

pub fn run_source(source: String) -> Result<(), Error> {
    run(source, &mut VirtualMachine::debug())
}

pub fn run_prompt() -> Result<(), Error> {
    let mut buf_reader = BufReader::new(io::stdin());
    let mut vm = VirtualMachine::default();
    loop {
        print!("> ");
        io::stdout().flush().context("could not flush stdout")?;

        let mut line = String::new();
        let read = buf_reader
            .read_line(&mut line)
            .context("could not read line from stdin")?;
        if read == 0 {
            break;
        }
        let _ = run(line, &mut vm);
    }
    Ok(())
}

pub fn run(source: String, vm: &mut VirtualMachine) -> Result<(), Error> {
    let reporter = Reporter::new(&source);
    let scanner = Scanner::new(&source);

    let mut compiler = Compiler::new(scanner, reporter, vm.storage());
    let chunk = compiler
        .compile()
        // .inspect(|chunk| println!("{chunk:?}"))
        .inspect_err(|err| reporter.report_unspanned(err))?;

    match vm.run(chunk) {
        Err(VirtualMachineError::Decode(err)) => {
            let err = anyhow::Error::new(err).context("Corrupted chunk");
            reporter.report_unspanned(&err);
            Err(err.into())
        }
        Err(VirtualMachineError::Runtime(err)) => {
            reporter.report(&err);
            Err(err.into())
        }
        Err(VirtualMachineError::Other(err)) => {
            reporter.report_unspanned(&err);
            Err(err.into())
        }
        Ok(()) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::VirtualMachine;

    /// Compile and run `source` under stress GC so the collector runs at every
    /// safe point — surfacing missed roots / use-after-free as Miri UB and
    /// exercising the unsafe downcast/drop paths the subprocess integration
    /// tests can't reach. Object teardown happens when the VM drops.
    fn run_ok(source: &str) {
        super::run(source.to_string(), &mut VirtualMachine::stress())
            .unwrap_or_else(|e| panic!("`{source}` failed: {e}"));
    }

    #[test]
    fn closures_capture_and_close() {
        run_ok(
            "fun make_adder(n) { fun adder(x) { return x + n; } return adder; }\n\
             var add5 = make_adder(5);\n\
             print add5(3);",
        );
    }

    #[test]
    fn classes_methods_init_and_fields() {
        run_ok(
            "class Counter {\n\
               init(start) { this.value = start; }\n\
               bump() { this.value = this.value + 1; return this.value; }\n\
             }\n\
             var c = Counter(10);\n\
             print c.bump();\n\
             var m = c.bump;\n\
             print m();",
        );
    }

    #[test]
    fn inheritance_and_super() {
        run_ok(
            "class Animal { speak() { return \"...\"; } }\n\
             class Dog < Animal { speak() { return super.speak() + \" woof\"; } }\n\
             print Dog().speak();",
        );
    }

    #[test]
    fn strings_and_natives() {
        run_ok("print clock() >= 0;\nprint \"a\" + \"b\" + \"c\";");
    }

    #[test]
    fn gc_keeps_reachable_instances() {
        // `keep` must survive ~50 collections while the loop churns garbage
        // instances and concat strings.
        run_ok(
            "class Box { init(v) { this.v = v; } }\n\
             var keep = Box(\"kept\");\n\
             for (var i = 0; i < 50; i = i + 1) {\n\
               var garbage = Box(\"g\" + \"arbage\");\n\
               garbage.extra = i;\n\
             }\n\
             print keep.v;",
        );
    }

    #[test]
    fn run_resets_state_after_error() {
        // A run that errors mid-expression must not poison the next run on the
        // same VM (the REPL reuses one). The first run errors after pushing
        // values and capturing an open upvalue; the second must still work.
        let mut vm = VirtualMachine::default();
        let first = super::run(
            "fun outer() { var captured = 99; fun inner() { return captured; } boom; }\nouter();"
                .to_string(),
            &mut vm,
        );
        assert!(
            first.is_err(),
            "first run should error on the undefined `boom`"
        );
        super::run(
            "{ var z = 5; fun g() { return z; } print g(); }".to_string(),
            &mut vm,
        )
        .expect("second run must succeed after the first errored");
    }

    #[test]
    fn large_local_scope_pops_correctly() {
        // 256 locals exceed a single PopN's u8 operand. If scope exit popped the
        // wrong count, the leftovers would alias a later block's slot-0 local, so
        // `f()` would call a number and error — run_ok asserts it succeeds.
        let mut src = String::from("{\n");
        for i in 0..256 {
            // Uninitialized (Nil) so this doesn't also overflow the constant pool.
            src.push_str(&format!("var v{i};\n"));
        }
        src.push_str("}\nfun ok() { return 1; }\n{ var f = ok; print f(); }\n");
        run_ok(&src);
    }

    #[test]
    fn gc_keeps_reachable_closures() {
        // `c`'s closure and its open/closed upvalue must survive while garbage
        // counters are collected.
        run_ok(
            "fun counter() { var n = 0; fun inc() { n = n + 1; return n; } return inc; }\n\
             var c = counter();\n\
             for (var i = 0; i < 30; i = i + 1) { var junk = counter(); }\n\
             print c();\n\
             print c();",
        );
    }
}
