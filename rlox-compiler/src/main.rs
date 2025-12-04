use rlox_compiler::{chunk::Chunk, opcode::OpCode, value::Value, vm::VirtualMachine};

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::default();
    let addr = chunk.add_constant(Value::new(1.2));
    chunk.write_with_line(OpCode::OpConstant(addr), 1);
    chunk.write_with_line(OpCode::NoOp, 1);
    chunk.write_with_line(OpCode::OpReturn, 2);
    println!("{chunk:?}");

    println!("Running VM...");
    let mut vm = VirtualMachine;
    vm.interpret(chunk)?;

    Ok(())
}
