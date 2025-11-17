use crate::compiler::ast_pass::Parameter;
use crate::builtins::{FunctionMap, Signature, add, expected};
use crate::errors::RuntimeError;
use crate::tokens::TokenType;
use crate::tokens::TokenType::U64;
use crate::value::{Value, u64};
use std::collections::HashMap;

macro_rules! mut_list_fn {
    (mut $list:ident, mut $args:ident => $body:expr) => {
        |self_val: Value, mut $args: Vec<Value>| -> Result<Value, RuntimeError> {
            match self_val {
                Value::List(mut $list) => $body,
                _ => Err(expected_a_list()),
            }
        }
    };
}

pub(crate) fn list_functions() -> FunctionMap {
    let mut list_functions: FunctionMap = HashMap::new();
    let functions = &mut list_functions;
    add(
        functions,
        "len",
        Signature::new(
            vec![],
            U64,
            mut_list_fn!(mut self_val, mut _args => Ok(u64(self_val.len() as u64))),
        ),
    );
    add(
        functions,
        "push",
        Signature::new(
            vec![Parameter::new("element", TokenType::Any)],
            U64,
            mut_list_fn!(mut list, mut args => {
                list.push(args.remove(0));
                Ok(Value::List(list))
            }),
        ),
    );
    add(
        functions,
        "remove",
        Signature::new(
            vec![Parameter::new("index", U64)],
            U64,
            mut_list_fn!(mut list, mut args => {
                let index = args.remove(0).cast_usize().unwrap();
                if index >= list.len() {
                    return Err(RuntimeError::IndexOutOfBounds(index, list.len()))
                }
                list.remove(index);
                Ok(Value::List(list))
            }),
        ),
    );
    list_functions
}

fn expected_a_list() -> RuntimeError {
    expected("list")
}
