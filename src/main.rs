
fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let chunk = crudlang::compiler::compile("\"1\"+\"2\"")?;
    chunk.disassemble();

    let result = crudlang::vm::interpret(chunk);
    println!("{:?}", result);
    Ok(())
}

