use vm::{chunk::Chunk, opcode::OpCode, value::Value, vm::VirtualMachine};

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::default();
    chunk.write_constant_with_line(123, Value::number(1.2));
    chunk.write_constant_with_line(123, Value::number(3.4));
    chunk.write_with_line(123, OpCode::Add);
    chunk.write_constant_with_line(123, Value::number(5.6));
    chunk.write_with_line(123, OpCode::Div);
    chunk.write_with_line(123, OpCode::Neg);
    chunk.write_with_line(123, OpCode::Return);
    println!("{chunk:?}");

    println!("Running VM...");
    let mut vm = VirtualMachine::default();
    vm.run(chunk)?;

    Ok(())
}
