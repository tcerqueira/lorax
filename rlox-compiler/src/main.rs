use rlox_compiler::{chunk::Chunk, opcode::OpCode, value::Value, vm::VirtualMachine};

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::default();
    chunk.write_constant_with_line(Value::new(1.2), 123);
    chunk.write_constant_with_line(Value::new(3.4), 123);
    chunk.write_with_line(OpCode::Add, 123);
    chunk.write_constant_with_line(Value::new(5.6), 123);
    chunk.write_with_line(OpCode::Div, 123);
    chunk.write_with_line(OpCode::Neg, 123);
    chunk.write_with_line(OpCode::Return, 123);
    println!("{chunk:?}");

    println!("Running VM...");
    let mut vm = VirtualMachine::default();
    vm.interpret(chunk)?;

    Ok(())
}
