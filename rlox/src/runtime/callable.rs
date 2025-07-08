use std::fmt::{Debug, Display};

use thiserror::Error;

use crate::{
    report::{Report, Span},
    runtime::{Interpreter, object::Object},
};

pub type CallableFn = Box<dyn Fn(&mut Interpreter, Vec<Object>) -> Result<Object, CallError>>;

pub struct Callable {
    pub name: Option<&'static str>,
    pub arity: u8,
    pub func: CallableFn,
}

impl Callable {
    pub fn arity(&self) -> u8 {
        self.arity
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, CallError> {
        (self.func)(interpreter, args)
    }
}

impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callable")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish_non_exhaustive()
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fun {}", self.name.unwrap_or("<anonymous>"))
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.func.as_ref(), other.func.as_ref())
    }
}

impl PartialOrd for Callable {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

#[derive(Debug, Error)]
#[error("[line {}:{}] {}", .span.line_start, .span.start, kind)]
pub struct CallError {
    span: Span,
    kind: CallErrorKind,
}

#[derive(Debug, Error)]
pub enum CallErrorKind {
    #[error("Expected {expected} arguments but found {found}")]
    Arity { expected: u8, found: usize },
    #[error("Object is not a callable")]
    NotCallable,
}

impl CallError {
    pub fn arity(span: Span, expected: u8, found: usize) -> Self {
        Self {
            span,
            kind: CallErrorKind::Arity { expected, found },
        }
    }

    pub fn not_callable(span: Span) -> Self {
        Self {
            span,
            kind: CallErrorKind::NotCallable,
        }
    }
}

impl Report for CallError {
    fn report(&self, _source: &str) {
        eprint!("{}", self.kind);
    }

    fn span(&self) -> &Span {
        &self.span
    }
}
