use thiserror::Error;

use crate::chunk::Chunk;

pub struct VirtualMachine;

#[derive(Debug, Error)]
pub enum Error {}

impl VirtualMachine {
    pub fn interpret(&mut self, _chunk: Chunk) -> Result<(), Error> {
        todo!()
    }
}
