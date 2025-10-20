// use crudlang::chunk::Chunk;
use crudlang::compiler::compile;
use crudlang::interpret;
// use crudlang::scanner::scan;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // let mut chunk = Chunk::new("main");
    // let constant = chunk.add_constant(1.2);
    // chunk.add(crudlang::opcode::OP_CONSTANT, 123);
    // chunk.add(constant as u16, 123);
    // chunk.add(crudlang::opcode::OP_NEGATE, 123);
    //
    // let constant = chunk.add_constant(3.4);
    // chunk.add(crudlang::opcode::OP_CONSTANT, 123);
    // chunk.add(constant as u16, 123);
    // chunk.add(crudlang::opcode::OP_ADD, 123);
    //
    // chunk.add(crudlang::opcode::OP_RETURN, 123);
    let chunk = compile("3<<2")?;
    chunk.disassemble();

    let result = interpret(chunk);
    println!("{:?}", result);
    Ok(())
}

