use rlox_compiler::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::default();
    chunk.write(OpCode::OpReturn);
    chunk.write(OpCode::OpReturn);
    println!("{chunk:?}");
}
