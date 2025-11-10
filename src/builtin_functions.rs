use crate::value::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use crate::errors::RuntimeError;

type MethodFn = fn(Value, Vec<Value>) -> Result<Value, RuntimeError>;
type MethodMap = HashMap<String, MethodFn>;
type MethodTable = HashMap<String, MethodMap>;

const METHODS: LazyLock<MethodTable> = LazyLock::new(|| {
    let mut table: MethodTable = HashMap::new();

    let mut string_methods: MethodMap = HashMap::new();
    string_methods.insert("len".to_string(), string_len);
    string_methods.insert("to_uppercase".to_string(), string_to_uppercase);
    string_methods.insert("contains".to_string(), string_contains);
    string_methods.insert("reverse".to_string(), string_reverse);

    table.insert("string".to_string(), string_methods);

    table
});

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

fn string_len(self_val: Value, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(Value::I64(s.len() as i64)),
        _ => Err(RuntimeError::ExpectedType("string".to_string())),
    }
}

fn string_to_uppercase(self_val: Value, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(RuntimeError::ExpectedType("string".to_string())),
    }
}

fn string_contains(self_val: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match (self_val, args.first()) {
        (Value::String(s), Some(Value::String(pat))) => {
            Ok(Value::Bool(s.contains(pat.as_str())))
        }
        _ => Err(RuntimeError::ExpectedType("string".to_string())),
    }
}
fn string_reverse(self_val: Value, _: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => {
            Ok(s.chars().rev().collect::<String>().into())
        }
        _ => Err(RuntimeError::ExpectedType("string".to_string())),
    }
}