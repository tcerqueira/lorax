use lasso::Spur;
use thiserror::Error;

use crate::enconding::LocalSlot;

/// Maximum number of locals live at once. Capped at `u8::MAX` to match the
/// chunk-constant limit and let any scope's pop count fit in a single `PopN`.
const MAX_LOCALS: usize = u8::MAX as usize;

#[derive(Debug, Clone, Copy)]
struct Local {
    name: Spur,
    depth: u32,
}

/// Locals form a stack; depths are monotonically non-decreasing front-to-back,
/// since inner scopes are fully drained before their enclosing scope ends.
#[derive(Default)]
pub struct Scopes {
    locals: Vec<Local>,
    depth: u32,
}

#[derive(Debug, Error)]
#[error("too many locals in scope (max {MAX_LOCALS})")]
pub struct TooManyLocals;

impl Scopes {
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }

    pub fn enter(&mut self) {
        self.depth += 1;
    }

    /// Drop all locals declared in the current scope. Returns how many were
    /// dropped — the caller emits matching `OpPop`s.
    pub fn exit(&mut self) -> usize {
        debug_assert!(self.depth > 0, "exit at global scope");
        let pop_count = self
            .locals
            .iter()
            .rev()
            .take_while(|l| l.depth == self.depth)
            .count();
        self.locals.truncate(self.locals.len() - pop_count);
        self.depth -= 1;
        pop_count
    }

    /// Push a new local at the current scope depth. Shadowing an existing
    /// local (in this or any enclosing scope) is deliberately allowed — see
    /// the `lorax` deviation from Lox spec, which permits `var a = a + 1;`
    /// using the previous binding.
    pub fn declare(&mut self, name: Spur) -> Result<LocalSlot, TooManyLocals> {
        if self.locals.len() >= MAX_LOCALS {
            return Err(TooManyLocals);
        }
        let slot = self.locals.len();
        self.locals.push(Local {
            name,
            depth: self.depth,
        });
        Ok(LocalSlot(slot as u8))
    }

    /// Resolve a name to the most recent local with that name, or `None` if
    /// no local matches (caller falls back to globals).
    pub fn resolve(&self, name: Spur) -> Option<LocalSlot> {
        self.locals
            .iter()
            .rposition(|l| l.name == name)
            .map(|i| i as u8)
            .map(LocalSlot)
    }
}

#[cfg(test)]
mod tests {
    use lasso::Rodeo;

    use super::*;

    fn make(names: &[&str]) -> (Rodeo, Vec<Spur>) {
        let mut r = Rodeo::new();
        let spurs = names.iter().map(|n| r.get_or_intern(n)).collect();
        (r, spurs)
    }

    #[test]
    fn declare_and_resolve_in_same_scope() {
        let (_r, s) = make(&["a"]);
        let mut scopes = Scopes::default();
        scopes.enter();
        let slot = scopes.declare(s[0]).unwrap();
        assert_eq!(slot, LocalSlot(0));
        assert_eq!(scopes.resolve(s[0]), Some(LocalSlot(0)));
    }

    #[test]
    fn resolve_returns_most_recent_shadow() {
        let (_r, s) = make(&["a"]);
        let mut scopes = Scopes::default();
        scopes.enter();
        scopes.declare(s[0]).unwrap();
        let second = scopes.declare(s[0]).unwrap();
        assert_eq!(scopes.resolve(s[0]), Some(second));
    }

    #[test]
    fn exit_pops_only_current_depth() {
        let (_r, s) = make(&["a", "b", "c"]);
        let mut scopes = Scopes::default();
        scopes.enter();
        scopes.declare(s[0]).unwrap();
        scopes.enter();
        scopes.declare(s[1]).unwrap();
        scopes.declare(s[2]).unwrap();
        assert_eq!(scopes.exit(), 2);
        assert_eq!(scopes.resolve(s[0]), Some(LocalSlot(0)));
        assert_eq!(scopes.resolve(s[1]), None);
        assert_eq!(scopes.resolve(s[2]), None);
    }

    #[test]
    fn unresolved_returns_none() {
        let (_r, s) = make(&["a"]);
        let mut scopes = Scopes::default();
        scopes.enter();
        assert_eq!(scopes.resolve(s[0]), None);
    }

    #[test]
    fn too_many_locals_errors() {
        let (_r, s) = make(&["x"]);
        let mut scopes = Scopes::default();
        scopes.enter();
        for _ in 0..MAX_LOCALS {
            scopes.declare(s[0]).unwrap();
        }
        assert!(scopes.declare(s[0]).is_err());
    }
}
