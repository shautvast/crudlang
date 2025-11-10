use crate::value::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use crate::builtins::string::string_methods;
use crate::errors::RuntimeError;

pub(crate) type MethodFn = fn(Value, Vec<Value>) -> Result<Value, RuntimeError>;
pub(crate) type MethodMap = HashMap<String, MethodFn>;
pub(crate) type MethodTable = HashMap<String, MethodMap>;

const METHODS: LazyLock<MethodTable> = LazyLock::new(|| {
    let mut table: MethodTable = HashMap::new();
    table.insert("string".to_string(), string_methods());
    table
});

pub(crate) fn insert(m: &mut MethodMap, name: &str, method: MethodFn) {
    m.insert(name.to_string(), method);
}

pub fn call_builtin(
    type_name: &str,
    method_name: &str,
    self_val: Value,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    METHODS
        .get(type_name)
        .and_then(|methods| methods.get(method_name))
        .ok_or_else(|| RuntimeError::FunctionNotFound(format!("{}.{}",type_name, method_name)))?
        (self_val, args)
}

