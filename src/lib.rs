use std::collections::HashMap;
use crate::scanner::scan;

pub mod ast_compiler;
pub mod bytecode_compiler;
pub mod chunk;
mod keywords;
pub mod scanner;
mod compiler_tests;
mod tokens;
mod value;
pub mod vm;

pub fn compile(src: &str) -> anyhow::Result<chunk::Chunk> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast= ast_compiler::compile(tokens)?;
    let bytecode = bytecode_compiler::compile("", &ast, &mut registry)?;
    Ok(bytecode)
}
