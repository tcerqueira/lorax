use crate::{chunk::Chunk, compiler::scopes::Scopes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FunctionKind {
    #[default]
    Script,
    Function,
}

struct CompileUnit {
    chunk: Chunk,
    scopes: Scopes,
    kind: FunctionKind,
}

impl CompileUnit {
    fn script() -> Self {
        Self::of(FunctionKind::Script)
    }

    fn of(kind: FunctionKind) -> Self {
        Self {
            chunk: Chunk::default(),
            scopes: Scopes::default(),
            kind,
        }
    }
}

pub struct Compilation {
    units: Vec<CompileUnit>,
}

impl Default for Compilation {
    fn default() -> Self {
        Self {
            units: vec![CompileUnit::script()],
        }
    }
}

impl Compilation {
    pub fn push_unit(&mut self, kind: FunctionKind) {
        self.units.push(CompileUnit::of(kind));
        // Slot 0 holds the callee at runtime; the script has none.
        if kind != FunctionKind::Script {
            self.scopes_mut()
                .reserve()
                .expect("reserving slot 0 in a fresh unit cannot overflow");
        }
    }

    /// Never pops the script unit at the bottom of the stack.
    pub fn pop_unit(&mut self) -> Chunk {
        assert!(self.units.len() > 1, "cannot pop the script unit");
        self.units.pop().expect("len checked above").chunk
    }

    pub fn chunk(&self) -> &Chunk {
        &self
            .units
            .last()
            .expect("always at least the global unit")
            .chunk
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self
            .units
            .last_mut()
            .expect("always at least the global unit")
            .chunk
    }

    pub fn scopes(&self) -> &Scopes {
        &self
            .units
            .last()
            .expect("always at least the global unit")
            .scopes
    }

    pub fn scopes_mut(&mut self) -> &mut Scopes {
        &mut self
            .units
            .last_mut()
            .expect("always at least the global unit")
            .scopes
    }

    pub fn at_global(&self) -> bool {
        let unit = self.units.last().expect("always at least the global unit");
        unit.kind == FunctionKind::Script && unit.scopes.is_root()
    }
}
