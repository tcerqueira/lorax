#![feature(formatting_options)]

use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use anyhow::Context;
use rlox_lexer::Scanner;
use rlox_report::{Error, Reporter};

use crate::vm::VirtualMachine;

pub mod chunk;
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

fn run(source: String, _vm: &mut VirtualMachine) -> Result<(), Error> {
    let reporter = Reporter::new(&source);
    let _tokens = Scanner::new(&source)
        .scan_tokens()
        .inspect_err(|errs| errs.iter().for_each(|e| reporter.report(e)))?;

    todo!();
}
