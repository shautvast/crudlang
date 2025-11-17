use crate::chunk::Chunk;
use crate::compiler::ast_pass::{Expression, Statement};
use crate::errors::CrudLangError::Platform;
use crate::errors::{CompilerErrorAtLine, CrudLangError};
use crate::symbol_builder::Symbol;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

mod builtins;
pub mod chunk;
pub(crate) mod compiler;
pub mod errors;
pub mod file_watch;
mod keywords;
pub mod repl;
mod symbol_builder;
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
            let tokens = compiler::scan_pass::scan(&source)?;
            let mut symbol_table = HashMap::new();
            match compiler::ast_pass::compile(Some(path), tokens, &mut symbol_table) {
                Ok(statements) => {
                    let path = path.strip_prefix(source_dir).unwrap().replace(".crud", "");

                    symbol_builder::build(&path, &statements, &mut symbol_table);
                    compiler::bytecode_pass::compile(Some(&path), &statements, &symbol_table, &mut registry)?;
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
    let tokens = compiler::scan_pass::scan(src)?;
    let mut symbol_table = HashMap::new();
    let ast = compiler::ast_pass::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    compiler::bytecode_pass::compile(None, &ast, &symbol_table, registry)?;
    Ok(())
}

pub fn compile(src: &str) -> Result<HashMap<String, Chunk>, CrudLangError> {
    let tokens = compiler::scan_pass::scan(src)?;
    let mut registry = HashMap::new();
    let mut symbol_table = HashMap::new();
    let ast = compiler::ast_pass::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    compiler::bytecode_pass::compile(None, &ast, &symbol_table, &mut registry)?;
    Ok(registry)
}

#[cfg(test)]
pub(crate) fn run(src: &str) -> Result<value::Value, CrudLangError> {
    let tokens = compiler::scan_pass::scan(src)?;
    let mut symbol_table = HashMap::new();
    let ast = compiler::ast_pass::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    let mut registry = HashMap::new();
    compiler::bytecode_pass::compile(None, &ast, &symbol_table, &mut registry)?;

    let registry = arc_swap::ArcSwap::from(std::sync::Arc::new(registry));
    vm::interpret(registry.load(), "main").map_err(CrudLangError::from)
}
