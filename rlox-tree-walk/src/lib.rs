use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use anyhow::Context;
use rlox_lexer::Scanner;
use rlox_report::Reporter;

use crate::{error::TreeWalkError, parsing::*, runtime::Interpreter};

use crate::{parsing::ast::AstArena, passes::resolver::Resolver};

pub mod error;
mod parsing;
mod passes;
pub mod runtime;

pub type Result<T> = ::std::result::Result<T, TreeWalkError>;

pub fn run_file(path: &Path) -> crate::Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read source file {}", path.display()))?;
    run(source, &mut Interpreter::new(), &mut AstArena::default())
}

pub fn run_prompt() -> crate::Result<()> {
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

    Resolver::new(interpreter, ast_arena).resolve(&program);

    interpreter
        .interpret(program, ast_arena)
        .inspect_err(|e| reporter.report(e))?;

    Ok(())
}
