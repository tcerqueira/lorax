use rlox_compiler::{chunk::Chunk, opcode::OpCode};

fn main() {
    let mut chunk = Chunk::default();
    chunk.write(OpCode::OpReturn);
    chunk.write(OpCode::NoOp);
    println!("{chunk:?}");
}
