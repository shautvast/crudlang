use crate::builtins::{MethodMap, Parameter, Signature, add, expected};
use crate::errors::RuntimeError;
use crate::tokens::TokenType::{StringType, U64};
use crate::value::{Value, bool, i64, string};
use regex::Regex;
use std::collections::HashMap;

pub(crate) fn string_methods() -> MethodMap {
    let mut string_methods: MethodMap = HashMap::new();
    let m = &mut string_methods;
    add(m, "len", Signature::new(vec![], U64, string_len));
    add(
        m,
        "to_uppercase",
        Signature::new(vec![], StringType, string_to_uppercase),
    );
    add(
        m,
        "to_lowercase",
        Signature::new(vec![], StringType, string_to_lowercase),
    );
    add(m, "contains", Signature::new(vec![], StringType, string_contains));
    add(m, "reverse", Signature::new(vec![], StringType, string_reverse));
    add(m, "trim", Signature::new(vec![], StringType, string_trim));
    add(
        m,
        "trim_start",
        Signature::new(vec![], StringType, string_trim_start),
    );
    add(m, "trim_end", Signature::new(vec![], StringType, string_trim_end));
    add(
        m,
        "replace_all",
        Signature::new(
            vec![
                Parameter::new("pattern", StringType),
                Parameter::new("replacement", StringType),
            ],
            StringType,
            string_replace_all,
        ),
    );
    string_methods
}

fn string_len(self_val: Value, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(i64(s.len() as i64)),
        _ => Err(expected_a_string()),
    }
}

fn string_to_uppercase(self_val: Value, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(string(s.to_uppercase())),
        _ => Err(expected_a_string()),
    }
}

fn string_to_lowercase(self_val: Value, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(string(s.to_lowercase())),
        _ => Err(expected_a_string()),
    }
}

fn string_contains(self_val: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    match (self_val, args.first()) {
        (Value::String(s), Some(Value::String(pat))) => Ok(bool(s.contains(pat.as_str()))),
        _ => Err(expected_a_string()),
    }
}

fn string_reverse(self_val: Value, _: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(s.chars().rev().collect::<String>().into()),
        _ => Err(expected_a_string()),
    }
}

fn string_trim(self_val: Value, _: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(string(s.trim())),
        _ => Err(expected_a_string()),
    }
}

fn string_trim_start(self_val: Value, _: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(string(s.trim_start())),
        _ => Err(expected_a_string()),
    }
}

fn string_trim_end(self_val: Value, _: Vec<Value>) -> Result<Value, RuntimeError> {
    match self_val {
        Value::String(s) => Ok(string(s.trim_end())),
        _ => Err(expected_a_string()),
    }
}
//TODO check arity in compiler (generically)
fn string_replace_all(receiver: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let pattern = if let Value::String(s) = &args[0] {
        Regex::new(s).map_err(|_| RuntimeError::IllegalArgumentException("Invalid regex".into()))?
    } else {
        return Err(RuntimeError::IllegalArgumentException(
            format!("Illegal pattern. Expected a string, but got {}", &args[0]).into(),
        ));
    };
    let replacement = if let Value::String(repl) = &args[1] {
        repl
    } else {
        return Err(RuntimeError::IllegalArgumentException(
            format!(
                "Illegal replacement. Expected a string but got {}",
                &args[1]
            )
            .into(),
        ));
    };
    match receiver {
        Value::String(ref str) => Ok(string(pattern.replace_all(str, replacement))),
        _ => Err(expected_a_string()),
    }
}

fn expected_a_string() -> RuntimeError {
    expected("string")
}
