use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{ExitCode, Termination},
};

use anyhow::Context;

use error::*;
use lexing::*;
use parsing::*;
use report::*;
use runtime::*;

mod error;
mod lexing;
mod parsing;
mod report;
mod runtime;

type Result<T> = ::std::result::Result<T, Error>;

fn main() -> crate::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.as_slice() {
        [_] => run_prompt(),
        [_, script_path] => run_file(Path::new(script_path)),
        _ => Err(Error::Cli),
    }
}

fn run_file(path: &Path) -> crate::Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read source file {}", path.display()))?;
    run(source)
}

fn run_prompt() -> crate::Result<()> {
    let mut buf_reader = BufReader::new(io::stdin());
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
        let _ = run(line);
    }
    Ok(())
}

fn run(source: String) -> crate::Result<()> {
    let reporter = Reporter::new(&source);
    let tokens = Scanner::new(&source)
        .scan_tokens()
        .inspect_err(|errs| errs.iter().for_each(|e| reporter.report(e)))?;

    let program = Parser::new(tokens)
        .parse()
        .inspect_err(|errs| errs.iter().for_each(|e| reporter.report(e)))?;

    Interpreter::new()
        .interpret(program)
        .inspect_err(|e| reporter.report(e))?;

    Ok(())
}

impl Termination for Error {
    fn report(self) -> ExitCode {
        match self {
            Error::Cli => ExitCode::from(64),
            Error::Parsing { .. } | Error::Lexing(_) => ExitCode::from(65),
            Error::Runtime(_) => ExitCode::from(70),
            Error::Other(_) => ExitCode::FAILURE,
        }
    }
}
