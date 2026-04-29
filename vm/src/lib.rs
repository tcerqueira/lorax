#![feature(formatting_options)]
#![feature(error_iter)]
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
    compiler::{CompileError, Compiler},
    vm::{VirtualMachine, VirtualMachineError},
};

pub mod chunk;
pub mod compiler;
pub(crate) mod debug;
pub(crate) mod enconding;
pub mod object;
pub mod opcode;
pub mod value;
pub mod vm;

pub fn run_file(path: &Path) -> Result<(), Error> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read source file {}", path.display()))?;
    run(source, &mut VirtualMachine::default())
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

    let mut compiler = Compiler::new(scanner);
    let chunk = compiler.compile().inspect_err(|err| match err {
        CompileError::Lexing(e) => reporter.report(e),
        CompileError::Parsing(e) => reporter.report(e),
        CompileError::Other(e) => reporter.report_unspanned(e),
    })?;

    match vm.run(chunk) {
        Err(VirtualMachineError::Decode(err)) => {
            let err = anyhow::Error::new(err).context("malformed chunk");
            reporter.report_unspanned(&err);
            Err(err.into())
        }
        Err(VirtualMachineError::Runtime(err)) => {
            reporter.report(&err);
            Err(err.into())
        }
        Ok(()) => Ok(()),
    }
}
