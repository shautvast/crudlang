use crate::chunk::Chunk;
use crate::errors::RuntimeError::Something;
use crate::errors::{RuntimeError, ValueError};
use crate::tokens::TokenType;
use crate::value::{Object, Value};
use arc_swap::Guard;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;
use crate::Registry;

pub struct Vm {
    ip: usize,
    stack: Vec<Value>,
    local_vars: HashMap<String, Value>,
    error_occurred: bool,
    pub(crate) registry: Arc<HashMap<String, Chunk>>,
}


pub fn interpret(
    registry: Guard<Arc<HashMap<String, Chunk>>>,
    function: &str,
) -> Result<Value, RuntimeError> {
    let chunk = registry.get(function).unwrap().clone();
    let mut vm = Vm::new(&registry);
    vm.run(&get_context(function), &chunk)
}

pub async fn interpret_async(
    registry: Guard<Arc<HashMap<String, Chunk>>>,
    function: &str,
    uri: &str,
    query_params: HashMap<String, String>,
    headers: HashMap<String, String>,
) -> Result<Value, RuntimeError> {
    let chunk = registry.get(function);
    if let Some(chunk) = chunk {
        let mut vm = Vm::new(&registry);
        vm.local_vars
            .insert("path".to_string(), Value::String(uri.into()));
        vm.local_vars
            .insert("query".to_string(), Value::Map(value_map(query_params)));
        vm.local_vars
            .insert("headers".to_string(), Value::Map(value_map(headers)));
        vm.run(&get_context(function), chunk)
    } else {
        Err(RuntimeError::FunctionNotFound(function.to_string()))
    }
}

fn value_map(strings: HashMap<String, String>) -> HashMap<Value, Value> {
    strings
        .into_iter()
        .map(|(k, v)| (Value::String(k.to_string()), Value::String(v.to_string())))
        .collect()
}

pub fn interpret_function(chunk: &Chunk, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut vm = Vm::new(& Arc::new(HashMap::new()));
    vm.run_function(chunk, args)
}

impl Vm {
    pub(crate) fn new(registry: &Arc<Registry>) -> Self {
        Self {
            ip: 0,
            stack: vec![],
            local_vars: HashMap::new(),
            error_occurred: false,
            registry: registry.clone(),
        }
    }

    fn run_function(&mut self, chunk: &Chunk, mut args: Vec<Value>) -> Result<Value, RuntimeError> {
        // arguments -> locals
        for (_, name) in chunk.vars.iter() {
            self.local_vars.insert(name.clone(), args.remove(0));
        }
        self.run("", chunk)
    }

    pub(crate) fn run(&mut self, context: &str, chunk: &Chunk) -> Result<Value, RuntimeError> {
        self.ip = 0;
        loop {
            if self.error_occurred {
                return Err(Something);
            }
            debug!("{:?}", self.stack);
            let opcode = chunk.code[self.ip];
            self.ip += 1;
            match opcode {
                OP_CONSTANT => {
                    let value = &chunk.constants[self.read(chunk)];
                    self.push(value.clone());
                }
                OP_ADD => binary_op(self, |a, b| a + b),
                OP_SUBTRACT => binary_op(self, |a, b| a - b),
                OP_MULTIPLY => binary_op(self, |a, b| a * b),
                OP_DIVIDE => binary_op(self, |a, b| a / b),
                OP_AND => binary_op(self, |a, b| {
                    if let (Value::Bool(a), Value::Bool(b)) = (a, b) {
                        Ok(Value::Bool(*a && *b))
                    } else {
                        Err(ValueError::Some("Cannot and"))
                    }
                }),
                OP_OR => binary_op(self, |a, b| {
                    if let (Value::Bool(a), Value::Bool(b)) = (a, b) {
                        Ok(Value::Bool(*a || *b))
                    } else {
                        Err(ValueError::Some("Cannot compare"))
                    }
                }),
                OP_NOT => unary_op(self, |a| !a),
                OP_BITAND => binary_op(self, |a, b| a & b),
                OP_BITOR => binary_op(self, |a, b| a | b),
                OP_BITXOR => binary_op(self, |a, b| a ^ b),
                OP_NEGATE => unary_op(self, |a| -a),
                OP_RETURN => {
                    debug!("return {:?}", self.stack);
                    return if self.stack.is_empty() {
                        Ok(Value::Void)
                    } else {
                        Ok(self.pop())
                    };
                }
                OP_SHL => binary_op(self, |a, b| a << b),
                OP_SHR => binary_op(self, |a, b| a >> b),
                OP_EQUAL => binary_op(self, |a, b| Ok(Value::Bool(a == b))),
                OP_GREATER => binary_op(self, |a, b| Ok(Value::Bool(a > b))),
                OP_GREATER_EQUAL => binary_op(self, |a, b| Ok(Value::Bool(a >= b))),
                OP_LESS => binary_op(self, |a, b| Ok(Value::Bool(a < b))),
                OP_LESS_EQUAL => binary_op(self, |a, b| Ok(Value::Bool(a <= b))),
                OP_PRINT => {
                    debug!("print {:?}", self.stack);
                    let v = self.pop();
                    println!("{}", v);
                }
                OP_DEFINE => {
                    let name = self.read_name(chunk);
                    let value = self.pop();
                    self.local_vars.insert(name, value);
                }
                OP_DEF_LIST => {
                    let len = self.read(chunk);
                    let mut list = vec![];
                    for _ in 0..len {
                        let value = self.pop();
                        list.push(value);
                    }
                    list.reverse();
                    self.push(Value::List(list));
                }
                OP_ASSIGN => {
                    let index = self.read(chunk);
                    let (var_type, name) = chunk.vars.get(index).unwrap();
                    let value = self.pop();
                    let value = match var_type {
                        TokenType::U32 => value.cast_u32()?,
                        TokenType::U64 => value.cast_u64()?,
                        TokenType::F32 => value.cast_f32()?,
                        TokenType::I32 => value.cast_i32()?,
                        _ => value,
                    };
                    self.local_vars.insert(name.to_string(), value);
                }
                OP_DEF_MAP => {
                    let len = self.read(chunk);
                    let mut map = HashMap::new();
                    for _ in 0..len {
                        let value = self.pop();
                        let key = self.pop();
                        map.insert(key, value);
                    }
                    self.push(Value::Map(map));
                }
                OP_GET => {
                    let var_index = self.read(chunk);
                    let (_, name_index) = chunk.vars.get(var_index).unwrap();
                    let value = self.local_vars.get(name_index).unwrap();
                    self.push(value.clone()); // not happy , take ownership, no clone
                }
                OP_LIST_GET => {
                    let index = self.pop();
                    let list = self.pop();
                    if let Value::List(list) = list {
                        self.push(list.get(index.cast_usize()?).cloned().unwrap())
                    }
                }
                OP_CALL_BUILTIN => {
                    let function_name_index = self.read(chunk);
                    let function_name = chunk.constants[function_name_index].to_string();
                    let function_type_index = self.read(chunk);
                    let receiver_type_name = chunk.constants[function_type_index].to_string();

                    let receiver = self.pop();
                    let num_args = self.read(chunk);
                    let mut args = vec![];
                    for _ in 0..num_args {
                        let arg = self.pop();
                        args.push(arg);
                    }
                    args.reverse();
                    let return_value = crate::builtins::call(&receiver_type_name, &function_name, receiver, args)?;
                    self.push(return_value);
                }
                OP_CALL => {
                    let function_name_index = self.read(chunk);
                    let num_args = self.read(chunk);

                    let mut args = vec![];
                    for _ in 0..num_args {
                        let arg = self.pop();
                        args.push(arg);
                    }
                    args.reverse();

                    let function_name = chunk.constants[function_name_index].to_string();
                    let function_chunk = self
                        .registry
                        .get(&function_name)
                        .or_else(|| self.registry.get(&format!("{}/{}", context, function_name)));

                    if function_chunk.is_none() {
                        let constructor = chunk.object_defs.get(&function_name);

                        if let Some(params) = constructor {
                            if params.len() != args.len() {
                                return Err(RuntimeError::IllegalArgumentsException(
                                    function_name,
                                    params.len(),
                                    args.len(),
                                ));
                            }

                            let mut fields = vec![];
                            params
                                .iter()
                                .zip(args)
                                .for_each(|(param, arg)| {
                                    fields.push((param.name.lexeme.clone(), arg))
                                });
                            let new_instance = Value::ObjectType(Box::new(Object {
                                definition: function_name,
                                fields,
                            }));
                            self.push(new_instance);
                        } else {
                            return Err(RuntimeError::FunctionNotFound(function_name));
                        }
                    } else {
                        let result = interpret_function(function_chunk.unwrap(), args)?;
                        self.push(result);
                    }
                }
                _ => {}
            }
        }
    }

    fn read(&mut self, chunk: &Chunk) -> usize {
        self.ip += 1;
        chunk.code[self.ip - 1] as usize
    }

    fn read_name(&mut self, chunk: &Chunk) -> String {
        let index = self.read(chunk);
        chunk.constants[index].to_string() //string??
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack
            .pop()
            .unwrap_or_else(|| Value::Error("Error occurred".to_string()))
    }
}

fn binary_op(vm: &mut Vm, op: impl Fn(&Value, &Value) -> Result<Value, ValueError> + Copy) {
    let b = vm.pop();
    let a = vm.pop();

    let result = op(&a, &b);
    match result {
        Ok(result) => vm.push(result),
        Err(e) => {
            vm.error_occurred = true;
            println!("Error: {} {:?} and {:?}", e, a, b);
        }
    }
}

fn unary_op(stack: &mut Vm, op: impl Fn(&Value) -> Result<Value, ValueError> + Copy) {
    let a = stack.pop();
    let result = op(&a);
    match result {
        Ok(result) => stack.push(result),
        Err(e) => panic!("Error: {:?} {:?}", e, a),
    }
}

pub(crate) fn get_context(path: &str) -> String {
    let mut parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        parts.truncate(parts.len() - 2);
    }
    parts.join("/")
}

pub const OP_CONSTANT: u16 = 1;
pub const OP_ADD: u16 = 2;
pub const OP_SUBTRACT: u16 = 3;
pub const OP_MULTIPLY: u16 = 4;
pub const OP_DIVIDE: u16 = 5;
pub const OP_NEGATE: u16 = 6;
pub const OP_PRINT: u16 = 7;
pub const OP_RETURN: u16 = 8;
pub const OP_CALL: u16 = 9;
pub const OP_DEF_FN: u16 = 10;
pub const OP_AND: u16 = 11;
pub const OP_OR: u16 = 12;
pub const OP_NOT: u16 = 13;
pub const OP_EQUAL: u16 = 14;
pub const OP_GREATER: u16 = 15;
pub const OP_LESS: u16 = 16;
pub const OP_NOT_EQUAL: u16 = 17;
pub const OP_GREATER_EQUAL: u16 = 18;
pub const OP_LESS_EQUAL: u16 = 19;
pub const OP_BITAND: u16 = 20;
pub const OP_BITOR: u16 = 21;
pub const OP_BITXOR: u16 = 22;
pub const OP_SHR: u16 = 23;
pub const OP_SHL: u16 = 24;
pub const OP_POP: u16 = 25;
pub const OP_DEFINE: u16 = 26; // may be obsolete already
pub const OP_GET: u16 = 27;
pub const OP_DEF_I32: u16 = 28;
pub const OP_DEF_I64: u16 = 29;
pub const OP_DEF_U32: u16 = 30;
pub const OP_DEF_U64: u16 = 31;
pub const OP_DEF_DATE: u16 = 32;
pub const OP_DEF_STRING: u16 = 33;
pub const OP_DEF_CHAR: u16 = 34;
pub const OP_DEF_BOOL: u16 = 35;
pub const OP_DEF_LIST: u16 = 36;
pub const OP_DEF_MAP: u16 = 37;
pub const OP_DEF_STRUCT: u16 = 38;
pub const OP_DEF_F32: u16 = 39;
pub const OP_DEF_F64: u16 = 40;
pub const OP_ASSIGN: u16 = 41;
pub const OP_LIST_GET: u16 = 42;
pub const OP_CALL_BUILTIN: u16 = 43;
