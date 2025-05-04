use super::Span;

pub trait Report {
    fn report(&self, source: &str);
    fn span(&self) -> &Span;
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
            "[line {}] Error '{}': ",
            span.line_start,
            span.slice(self.src)
        );
        error.report(self.src);
        eprintln!()
    }
}
