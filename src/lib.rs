use crate::chunk::Chunk;
use crate::errors::Error;
use crate::errors::Error::Platform;
use crate::scanner::scan;
use crate::value::Value;
use crate::vm::interpret;
use std::collections::HashMap;
use std::fs;
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

pub fn compile_sourcedir(source_dir: &str) -> Result<HashMap<String, Chunk>, Error> {
    let mut registry = HashMap::new();

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        if path.ends_with(".crud") {
            print!("compiling {:?}: ", path);
            let source = fs::read_to_string(path).map_err(map_underlying())?;
            let tokens = scan(&source)?;
            match ast_compiler::compile(Some(&path), tokens) {
                Ok(statements) => {
                    let path = path.strip_prefix("source/").unwrap().replace(".crud", "");
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

pub fn map_underlying() -> fn(std::io::Error) -> Error {
    |e| Platform(e.to_string())
}

pub fn compile(src: &str) -> Result<HashMap<String, Chunk>, Error> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast = ast_compiler::compile(None, tokens)?;
    bytecode_compiler::compile(None, &ast, &mut registry)?;
    Ok(registry)
}

fn run(src: &str) -> Result<Value, Error> {
    let tokens = scan(src)?;
    let mut registry = HashMap::new();
    let ast = ast_compiler::compile(None, tokens)?;
    bytecode_compiler::compile(None, &ast, &mut registry)?;
    interpret(&registry, "main").map_err(Error::from)
}
