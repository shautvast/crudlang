use std::collections::HashMap;
use crate::builtins::builtin_functions::{insert, MethodMap};
use crate::errors::RuntimeError;
use crate::value::Value;

pub(crate) fn string_methods() -> MethodMap {
    let mut string_methods: MethodMap = HashMap::new();
    let m = &mut string_methods;
    insert(m, "len", string_len);
    insert(m, "to_uppercase", string_to_uppercase);
    insert(m, "contains", string_contains);
    insert(m, "reverse", string_reverse);
    string_methods
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