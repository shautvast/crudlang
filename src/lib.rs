use crate::compiler::asm_pass::AsmChunk;
use crate::compiler::ast_pass::{Expression, Statement};
use crate::errors::CompilerErrorAtLine;
use crate::symbol_builder::Symbol;
use std::collections::HashMap;

mod builtins;
pub mod compiler;
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
pub(crate) type AsmRegistry = HashMap<String, AsmChunk>;
