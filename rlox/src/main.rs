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

use crate::parsing::ast::AstArena;

mod error;
mod lexing;
mod parsing;
mod report;
mod resolver;
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
    run(source, &mut Interpreter::new(), &mut AstArena::default())
}

fn run_prompt() -> crate::Result<()> {
    let mut buf_reader = BufReader::new(io::stdin());
    let mut ast_arena = AstArena::default();
    let mut interpreter = Interpreter::new();
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
        let _ = run(line, &mut interpreter, &mut ast_arena);
    }
    Ok(())
}

fn run(
    source: String,
    interpreter: &mut Interpreter,
    ast_arena: &mut AstArena,
) -> crate::Result<()> {
    let reporter = Reporter::new(&source);
    let tokens = Scanner::new(&source)
        .scan_tokens()
        .inspect_err(|errs| errs.iter().for_each(|e| reporter.report(e)))?;

    let program = Parser::new(ast_arena, tokens)
        .parse()
        .inspect_err(|errs| errs.iter().for_each(|e| reporter.report(e)))?;

    interpreter
        .interpret(program, ast_arena)
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
