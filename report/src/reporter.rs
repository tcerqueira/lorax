use crate::Spanned;
use crate::error::{LexingError, ParsingError, RuntimeError};

pub trait Report: Spanned {
    fn report(&self, source: &str);
}

pub struct Reporter<'s> {
    src: &'s str,
}

impl<'s> Reporter<'s> {
    pub fn new(src: &'s str) -> Self {
        Self { src }
    }

    pub fn report(&self, error: &impl Report) {
        let span = error.span();
        eprint!(
            "[line {:>4}] Error '{}': ",
            span.line_start,
            span.slice(self.src)
        );
        error.report(self.src);
        eprintln!()
    }

    pub fn report_unspanned(&self, error: &anyhow::Error) {
        for cause in error.chain() {
            if let Some(e) = cause.downcast_ref::<LexingError>() {
                self.report(e);
            } else if let Some(e) = cause.downcast_ref::<ParsingError>() {
                self.report(e);
            } else if let Some(e) = cause.downcast_ref::<RuntimeError>() {
                self.report(e);
            } else {
                eprintln!("Error: {cause}");
            }
        }
    }
}
