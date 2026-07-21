use std::io::Write;

use crate::Spanned;
use crate::error::{LexingError, ParsingError, RuntimeError};

pub trait Report: Spanned {
    fn report(&self, source: &str, w: &mut dyn Write);
}

pub struct Reporter<'s, 'w> {
    src: &'s str,
    err: &'w mut dyn Write,
}

impl<'s, 'w> Reporter<'s, 'w> {
    pub fn new(src: &'s str, err: &'w mut dyn Write) -> Self {
        Self { src, err }
    }

    pub fn report(&mut self, error: &impl Report) {
        let span = error.span();
        let _ = write!(
            self.err,
            "[line {:>4}] Error '{}': ",
            span.line_start,
            span.slice(self.src)
        );
        error.report(self.src, &mut *self.err);
        let _ = writeln!(self.err);
    }

    pub fn report_unspanned(&mut self, error: &anyhow::Error) {
        for cause in error.chain() {
            if let Some(e) = cause.downcast_ref::<LexingError>() {
                self.report(e);
            } else if let Some(e) = cause.downcast_ref::<ParsingError>() {
                self.report(e);
            } else if let Some(e) = cause.downcast_ref::<RuntimeError>() {
                self.report(e);
            } else {
                let _ = writeln!(self.err, "Error: {cause}");
            }
        }
    }
}
