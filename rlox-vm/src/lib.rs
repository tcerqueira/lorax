#![feature(formatting_options)]
#![feature(error_iter)]

use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use anyhow::Context;
use rlox_lexer::Scanner;
use rlox_report::{Error, Reporter};

use crate::{
    compiler::Compiler,
    vm::{VirtualMachine, VirtualMachineError},
};

pub mod chunk;
pub mod compiler;
pub(crate) mod debug;
pub(crate) mod enconding;
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

fn run(source: String, vm: &mut VirtualMachine) -> Result<(), Error> {
    let _reporter = Reporter::new(&source);
    let scanner = Scanner::new(&source);

    let mut compiler = Compiler::new(scanner);
    let chunk = compiler.compile()?;

    match vm.run(chunk) {
        Err(VirtualMachineError::Decode(err)) => {
            Err(anyhow::Error::new(err).context("malformed chunk").into())
        }
        Err(VirtualMachineError::Runtime(err)) => Err(err.into()),
        Ok(()) => Ok(()),
    }
}
