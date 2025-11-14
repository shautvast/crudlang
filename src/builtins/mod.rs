mod string;

use crate::builtins::string::string_methods;
use crate::errors::{CompilerError, RuntimeError};
use crate::tokens::TokenType;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use crate::ast_compiler::Parameter;

pub(crate) struct Signature {
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) return_type: TokenType,
    pub(crate) function: MethodFn,
}

impl Signature {
    pub(crate) fn new(
        parameters: Vec<Parameter>,
        return_type: TokenType,
        function: MethodFn,
    ) -> Self {
        Self {
            parameters,
            return_type,
            function,
        }
    }

    pub(crate) fn arity(&self) -> usize {
        self.parameters.len()
    }
}

pub(crate) type MethodFn = fn(Value, Vec<Value>) -> Result<Value, RuntimeError>;
pub(crate) type MethodMap = HashMap<String, Signature>;
pub(crate) type MethodTable = HashMap<String, MethodMap>;

static METHODS: LazyLock<MethodTable> = LazyLock::new(|| {
    let mut table: MethodTable = HashMap::new();
    table.insert("string".to_string(), string_methods());
    table
});

pub(crate) fn add(m: &mut MethodMap, name: &str, method: Signature) {
    m.insert(name.to_string(), method);
}

pub(crate) fn lookup(type_name: &str, method_name: &str) -> Result<&'static Signature, CompilerError> {
     METHODS
        .get(type_name)
        .and_then(|methods| methods.get(method_name))
        .ok_or_else(|| CompilerError::FunctionNotFound(format!("{}.{}", type_name, method_name)))
}

pub(crate) fn call(
    type_name: &str,
    method_name: &str,
    self_val: Value,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    (lookup(type_name,method_name).map_err(|e|RuntimeError::FunctionNotFound(e.to_string()))?.function)(self_val, args)
}

pub(crate) fn expected(expected_type: &str) -> RuntimeError {
    RuntimeError::ExpectedType(expected_type.to_string())
}
