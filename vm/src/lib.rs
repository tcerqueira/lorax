#![feature(formatting_options)]
#![feature(error_iter)]
#![feature(ptr_metadata)]
#![feature(arbitrary_self_types)]
#![feature(if_let_guard)]

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
pub(crate) mod enconding;
pub mod object;
pub mod opcode;
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
