use crudlang::{ast_compiler, chunk};
use crudlang::bytecode_compiler::compile;
use crudlang::scanner::scan;
use crudlang::vm::{interpret, Vm};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let tokens = scan(r#"let a = 42
print a"#);
    match ast_compiler::compile(tokens) {
        Ok(statements) => {
            println!("{:?}", statements);
            let chunk = compile(statements)?;
            chunk.disassemble();
            interpret(chunk);
        }
        Err(e) => {
            println!("{}", e)
        }
    }

    // println!("{}",expression.infer_type());

    // let chunk = crudlang::compiler::compile(
    //     r#"let a ="hello " + 42"#,
    // );
    // match chunk {
    //     Err(e) => {
    //         println!("{}", e);
    //         return Ok(());
    //     }
    //     Ok(chunk) => {
    //         chunk.disassemble();
    //
    //         let result = crudlang::vm::interpret(chunk)?;
    //         println!("{}", result);
    //     }
    // }

    Ok(())
}
