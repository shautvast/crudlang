fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let chunk = crudlang::compiler::compile(
        r#"let a:bool = 42"#,
    );
    match chunk {
        Err(e) => {
            println!("{}", e);
            return Ok(());
        }
        Ok(chunk) => {
            chunk.disassemble();

            let result = crudlang::vm::interpret(chunk)?;
            println!("{}", result);
        }
    }

    Ok(())
}
