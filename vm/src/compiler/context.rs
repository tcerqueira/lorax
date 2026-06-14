use crate::{chunk::Chunk, compiler::scopes::Scopes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FunctionKind {
    #[default]
    Script,
    #[expect(dead_code)]
    Function,
}

struct Target {
    chunk: Chunk,
    scopes: Scopes,
    kind: FunctionKind,
}

impl Target {
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

pub struct LexicalContext {
    targets: Vec<Target>,
}

impl Default for LexicalContext {
    fn default() -> Self {
        Self {
            targets: vec![Target::script()],
        }
    }
}

impl LexicalContext {
    pub fn chunk(&self) -> &Chunk {
        &self
            .targets
            .last()
            .expect("always at least the global target")
            .chunk
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self
            .targets
            .last_mut()
            .expect("always at least the global target")
            .chunk
    }

    pub fn scopes(&self) -> &Scopes {
        &self
            .targets
            .last()
            .expect("always at least the global target")
            .scopes
    }

    pub fn scopes_mut(&mut self) -> &mut Scopes {
        &mut self
            .targets
            .last_mut()
            .expect("always at least the global target")
            .scopes
    }

    pub fn at_global(&self) -> bool {
        let target = self
            .targets
            .last()
            .expect("always at least the global target");
        target.kind == FunctionKind::Script && target.scopes.is_root()
    }
}
