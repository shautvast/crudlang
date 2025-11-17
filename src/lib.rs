use crate::ast_compiler::{Expression, Statement};
use crate::chunk::Chunk;
use crate::errors::CrudLangError::Platform;
use crate::errors::{CompilerErrorAtLine, CrudLangError};
use crate::scanner::scan;
use crate::symbol_builder::Symbol;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;
use crate::value::Value::Void;

pub mod ast_compiler;
mod builtins;
pub mod bytecode_compiler;
pub mod chunk;
mod compiler_tests;
pub mod errors;
pub mod file_watch;
mod keywords;
pub mod repl;
pub mod scanner;
mod symbol_builder;
mod tokens;
mod value;
pub mod vm;

pub(crate) type SymbolTable = HashMap<String, Symbol>;
pub(crate) type Expr = Result<Expression, CompilerErrorAtLine>;
pub(crate) type Stmt = Result<Statement, CompilerErrorAtLine>;
pub(crate) type Registry = HashMap<String, Chunk>;

pub fn compile_sourcedir(source_dir: &str) -> Result<HashMap<String, Chunk>, CrudLangError> {
    let mut registry = HashMap::new();

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        if path.ends_with(".crud") {
            print!("-- Compiling {} -- ", path);
            let source = fs::read_to_string(path).map_err(map_underlying())?;
            let tokens = scan(&source)?;
            let mut symbol_table = HashMap::new();
            match ast_compiler::compile(Some(path), tokens, &mut symbol_table) {
                Ok(statements) => {
                    let path = path.strip_prefix(source_dir).unwrap().replace(".crud", "");

                    symbol_builder::build(&path, &statements, &mut symbol_table);
                    bytecode_compiler::compile(
                        Some(&path),
                        &statements,
                        &symbol_table,
                        &mut registry,
                    )?;
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
        }
    }
    println!();
    Ok(registry)
}

pub fn map_underlying() -> fn(std::io::Error) -> CrudLangError {
    |e| Platform(e.to_string())
}

pub fn recompile(src: &str, registry: &mut HashMap<String, Chunk>) -> Result<(), CrudLangError> {
    let tokens = scan(src)?;
    let mut symbol_table = HashMap::new();
    let ast = ast_compiler::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    bytecode_compiler::compile(None, &ast, &symbol_table, registry)?;
    Ok(())
}

pub fn compile(src: &str) -> Result<HashMap<String, Chunk>, CrudLangError> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let mut symbol_table = HashMap::new();
    let ast = ast_compiler::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    bytecode_compiler::compile(None, &ast, &symbol_table, &mut registry)?;
    Ok(registry)
}

#[cfg(test)]
pub(crate) fn run(src: &str) -> Result<value::Value, CrudLangError> {
        let tokens = scan(src)?;
        let mut symbol_table = HashMap::new();
        let ast = ast_compiler::compile(None, tokens, &mut symbol_table)?;
        symbol_builder::build("", &ast, &mut symbol_table);
        let mut registry = HashMap::new();
        bytecode_compiler::compile(None, &ast, &symbol_table, &mut registry)?;

        let registry = arc_swap::ArcSwap::from(std::sync::Arc::new(registry));
        vm::interpret(registry.load(), "main").map_err(CrudLangError::from)
}
