use rlox_compiler::{chunk::Chunk, opcode::OpCode, value::Value};

fn main() {
    let mut chunk = Chunk::default();

    let addr = chunk.add_constant(Value::new(1.2));
    chunk.write(OpCode::OpConstant(addr));

    chunk.write(OpCode::OpReturn);
    chunk.write(OpCode::NoOp);

    println!("{chunk:?}");
}
