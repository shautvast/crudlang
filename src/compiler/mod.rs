use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;
use crate::{compiler, symbol_builder, AsmRegistry};
use crate::compiler::asm_pass::AsmChunk;
use crate::errors::CrudLangError;
use crate::errors::CrudLangError::Platform;

mod compiler_tests;
pub mod scan_pass;
pub mod ast_pass;
pub mod tokens;
pub mod asm_pass;

pub fn compile_sourcedir(source_dir: &str) -> Result<HashMap<String, AsmChunk>, CrudLangError> {
    let mut asm_registry = AsmRegistry::new();

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        if path.ends_with(".crud") {
            print!("-- Compiling {} -- ", path);
            let source = fs::read_to_string(path).map_err(map_underlying())?;
            let tokens = scan_pass::scan(&source)?;
            let mut symbol_table = HashMap::new();
            match ast_pass::compile(Some(path), tokens, &mut symbol_table) {
                Ok(statements) => {
                    let path = path.strip_prefix(source_dir).unwrap().replace(".crud", "");

                    symbol_builder::build(&path, &statements, &mut symbol_table);
                    asm_pass::compile(Some(&path), &statements, &symbol_table, &mut asm_registry)?;
                }
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
        }
    }

    Ok(asm_registry)
}

pub fn map_underlying() -> fn(std::io::Error) -> CrudLangError {
    |e| Platform(e.to_string())
}


pub fn compile(src: &str) -> Result<HashMap<String, AsmChunk>, CrudLangError> {
    let tokens = compiler::scan_pass::scan(src)?;
    let mut asm_registry = HashMap::new();
    let mut symbol_table = HashMap::new();
    let ast = compiler::ast_pass::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    asm_pass::compile(None, &ast, &symbol_table, &mut asm_registry)?;
    Ok(asm_registry)
}

#[cfg(test)]
pub(crate) fn run(src: &str) -> Result<crate::value::Value, CrudLangError> {
    let tokens = compiler::scan_pass::scan(src)?;
    let mut symbol_table = HashMap::new();
    let ast = compiler::ast_pass::compile(None, tokens, &mut symbol_table)?;
    symbol_builder::build("", &ast, &mut symbol_table);
    let mut asm_registry = HashMap::new();
    asm_pass::compile(None, &ast, &symbol_table, &mut asm_registry)?;
    let registry = arc_swap::ArcSwap::from(std::sync::Arc::new(asm_registry));
    crate::vm::interpret(registry.load(), "main").map_err(CrudLangError::from)
}
