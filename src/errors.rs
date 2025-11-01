use thiserror::Error;
use crate::tokens::{Token, TokenType};

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error(transparent)]
    Compiler(#[from] CompilerError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("Platform error {0}")]
    Platform(String),
}


#[derive(Error, Debug, PartialEq)]
pub enum CompilerError {
    #[error("Compilation failed")]
    Failure,
    #[error("Too many parameters")]
    TooManyParameters,
    #[error("Expected {0}")]
    Expected(&'static str),
    #[error("unexpected indent level {0} vs expected {1}")]
    UnexpectedIndent(usize,usize),
    #[error("Type mismatch at line {0}: {1}")]
    TypeError(usize, Box<CompilerError>),
    #[error("Uninitialized variables are not allowed.")]
    UninitializedVariable,
    #[error("Incompatible types. Expected {0}, found {1}")]
    IncompatibleTypes(TokenType, TokenType),
    #[error("Error parsing number {0}")]
    ParseError(String),
    #[error("Undeclared variable: {0:?}")]
    UndeclaredVariable(Token),
    #[error("Unexpected identifier at line {0}")]
    UnexpectedIdentifier(usize),
    #[error("Unterminated {0} at line {1}")]
    Unterminated(&'static str, usize),
    #[error("Illegal char length for {0} at line {1}")]
    IllegalCharLength(String, usize),
    #[error("Unexpected type {0}")]
    UnexpectedType(TokenType)
}

#[derive(Error, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("Error while executing")]
    Value(#[from] ValueError),
    #[error("Error occurred")]
    Something,
    #[error("Expected {0}, got {1}")]
    Expected(&'static str, &'static str),
}

#[derive(Error, Debug, PartialEq)]
pub enum ValueError {
    #[error("{0}")]
    CannotAnd(&'static str),
    #[error("{0}")]
    Some(&'static str),
}