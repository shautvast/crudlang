use crate::scanner::scan;
use crate::value::Value;
use crate::vm::interpret;
use std::collections::HashMap;
use crate::errors::Error;

pub mod ast_compiler;
pub mod bytecode_compiler;
pub mod chunk;
mod compiler_tests;
mod keywords;
pub mod scanner;
mod tokens;
mod value;
pub mod vm;
pub mod errors;

pub fn compile(src: &str) -> Result<HashMap<String, chunk::Chunk>, Error> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast = ast_compiler::compile(tokens)?;
    bytecode_compiler::compile(None, &ast, &mut registry)?;
    Ok(registry)
}

fn run(src: &str) -> Result<Value, Error> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast = ast_compiler::compile(tokens)?;
    bytecode_compiler::compile(None, &ast, &mut registry)?;
    interpret(&registry, "main").map_err(Error::from)
}
