use crate::builtins::{FunctionMap, Signature, add};
use crate::compiler::tokens::TokenType::DateTime;
use crate::errors::RuntimeError;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

pub(crate) static GLOBAL_FUNCTIONS: LazyLock<FunctionMap> = LazyLock::new(|| {
    let mut global_functions: FunctionMap = HashMap::new();
    let functions = &mut global_functions;
    add(functions, "now", Signature::new(vec![], DateTime, now));

    global_functions
});

fn now(_self_val: Value, _args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::DateTime(Box::new(chrono::DateTime::from(
        chrono::Utc::now(),
    ))))
}
