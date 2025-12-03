use rlox_compiler::{chunk::Chunk, opcode::OpCode, value::Value};

fn main() {
    let mut chunk = Chunk::default();

    let addr = chunk.add_constant(Value::new(1.2));
    chunk.write_with_line(OpCode::OpConstant(addr), 1);

    chunk.write(OpCode::OpReturn);
    chunk.write_with_line(OpCode::NoOp, 123);

    println!("{chunk:?}");
}
