use crate::chunk::Chunk;
use crate::errors::CrudLangError;
use crate::errors::CrudLangError::Platform;
use crate::scanner::scan;
use crate::value::Value;
use crate::vm::interpret;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use arc_swap::ArcSwap;
use walkdir::WalkDir;

pub mod ast_compiler;
pub mod bytecode_compiler;
pub mod chunk;
mod compiler_tests;
pub mod errors;
mod keywords;
pub mod scanner;
mod tokens;
mod value;
pub mod vm;
pub mod repl;
pub mod file_watch;

pub fn compile_sourcedir(source_dir: &str) -> Result<HashMap<String, Chunk>, CrudLangError> {
    let mut registry = HashMap::new();

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        if path.ends_with(".crud") {
            print!("-- Compiling ");
            let source = fs::read_to_string(path).map_err(map_underlying())?;
            let tokens = scan(&source)?;
            match ast_compiler::compile(Some(&path), tokens) {
                Ok(statements) => {
                    println!("{}",path);
                    let path = path.strip_prefix(source_dir).unwrap().replace(".crud", "");
                    bytecode_compiler::compile(Some(&path), &statements, &mut registry)?;
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
            println!();
        }
    }
    Ok(registry)
}

pub fn map_underlying() -> fn(std::io::Error) -> CrudLangError {
    |e| Platform(e.to_string())
}

pub fn compile(src: &str) -> Result<HashMap<String, Chunk>, CrudLangError> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast = ast_compiler::compile(None, tokens)?;
    bytecode_compiler::compile(None, &ast, &mut registry)?;
    Ok(registry)
}

fn run(src: &str) -> Result<Value, CrudLangError> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast = ast_compiler::compile(None, tokens)?;
    bytecode_compiler::compile(None, &ast, &mut registry)?;
    let registry = ArcSwap::from(Arc::new(registry));
    interpret(registry.load(), "main").map_err(CrudLangError::from)
}
