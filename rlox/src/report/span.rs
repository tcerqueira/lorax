#![allow(dead_code)]

pub trait Spanned {
    fn span(&self) -> Span;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line_start: u32,
    pub line_end: u32,
}

impl Span {
    pub fn join(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line_start: self.line_start.min(other.line_start),
            line_end: self.line_end.max(other.line_end),
        }
    }

    pub fn slice<'s>(&self, s: &'s str) -> &'s str {
        &s[self.start..self.end]
    }
}

impl Spanned for &Span {
    fn span(&self) -> Span {
        (*self).clone()
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            line_start: 1,
            line_end: 1,
        }
    }
}
