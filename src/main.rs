use crudlang::ast_compiler;
use crudlang::bytecode_compiler::compile;
use crudlang::scanner::scan;
use crudlang::vm::interpret;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let tokens = scan(
        r#"
fn main(a: string) -> u32:
    a + 42
let text = "hello "
main(text)"#,
    );
    println!("{:?}", tokens);
    match ast_compiler::compile(tokens) {
        Ok(statements) => {
            println!("{:?}", statements);
            let chunk = compile(&statements)?;
            chunk.disassemble();
            println!("{}",interpret(&chunk)?);
        }
        Err(e) => {
            println!("{}", e)
        }
    }

    Ok(())
}
