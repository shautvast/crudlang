use crate::chunk::Chunk;
use crate::value::Value;
use anyhow::anyhow;
use std::collections::HashMap;
use tracing::debug;

macro_rules! define_var {
    ($self:ident, $variant:ident, $chunk:ident) => {{
        let name = $self.read_name($chunk);
        let value = $self.pop();
        if let Value::$variant(_) = value {
            $self.local_vars.insert(name, value);
        } else {
            return Err(anyhow!(
                concat!("Expected ", stringify!($variant), ", got {:?}"),
                value
            ));
        }
    }};
}

pub struct Vm<'a> {
    ip: usize,
    stack: Vec<Value>,
    local_vars: HashMap<String, Value>,
    error_occurred: bool,
    registry: &'a HashMap<String, Chunk>,
}

pub async fn interpret(registry: &HashMap<String, Chunk>, function: &str) -> anyhow::Result<Value> {
    let chunk = registry.get(function).unwrap().clone();
    let mut vm = Vm {
        ip: 0,
        stack: vec![],
        local_vars: HashMap::new(),
        error_occurred: false,
        registry,
    };
    vm.run(&chunk, vec![])
}

pub fn interpret_function(chunk: &Chunk, args: Vec<Value>) -> anyhow::Result<Value> {
    let mut vm = Vm {
        ip: 0,
        stack: vec![],
        local_vars: HashMap::new(),
        error_occurred: false,
        registry: &HashMap::new(),
    };
    vm.run(chunk, args)
}

impl <'a> Vm<'a> {
    fn run(&mut self, chunk: &Chunk, args: Vec<Value>) -> anyhow::Result<Value> {
        for arg in args {
            self.push(arg);
        }
        loop {
            if self.error_occurred {
                return Err(anyhow!("Error occurred"));
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
                        Err(anyhow!("Cannot and"))
                    }
                }),
                OP_OR => binary_op(self, |a, b| {
                    if let (Value::Bool(a), Value::Bool(b)) = (a, b) {
                        Ok(Value::Bool(*a || *b))
                    } else {
                        Err(anyhow!("Cannot compare"))
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
                OP_DEF_I32 => define_var!(self, I32, chunk),
                OP_DEF_I64 => define_var!(self, I64, chunk),
                OP_DEF_U32 => define_var!(self, U32, chunk),
                OP_DEF_U64 => define_var!(self, U64, chunk),
                OP_DEF_F32 => define_var!(self, F32, chunk),
                OP_DEF_F64 => define_var!(self, F64, chunk),
                OP_DEF_STRING => define_var!(self, String, chunk),
                OP_DEF_CHAR => define_var!(self, Char, chunk),
                OP_DEF_BOOL => define_var!(self, Bool, chunk),
                OP_DEF_DATE => define_var!(self, Date, chunk),
                OP_DEF_LIST => {
                    let name = self.read_name(chunk);
                    let len = self.read(chunk);
                    let mut list = vec![];
                    for _ in 0..len {
                        let value = self.pop();
                        list.push(value);
                    }
                    self.local_vars.insert(name, Value::List(list));
                }
                OP_DEF_MAP => define_var!(self, Map, chunk),
                OP_DEF_STRUCT => define_var!(self, Struct, chunk),
                OP_GET => {
                    let name = self.read_name(chunk);
                    let value = self.local_vars.get(&name).unwrap();
                    self.push(value.clone()); // not happy
                    debug!("after get {:?}", self.stack);
                }
                OP_CALL => {
                    let function_name_index = self.read(chunk);
                    let num_args = self.read(chunk);

                    let mut args = vec![];
                    for _ in 0..num_args {
                        let arg = self.pop();
                        args.push(arg);
                    }
                    // args.reverse();

                    let function_name = chunk.constants[function_name_index].to_string();
                    let function_chunk = self.registry.get(&function_name).unwrap();
                    let result = interpret_function(function_chunk, args)?;
                    self.push(result);
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

fn binary_op(vm: &mut Vm, op: impl Fn(&Value, &Value) -> anyhow::Result<Value> + Copy) {
    let b = vm.pop();
    let a = vm.pop();

    let result = op(&a, &b);
    match result {
        Ok(result) => vm.push(result),
        Err(e) => {
            println!("Error: {} {:?} and {:?}", e.to_string(), a, b);
            vm.error_occurred = true;
        }
    }
}

fn unary_op(stack: &mut Vm, op: impl Fn(&Value) -> anyhow::Result<Value> + Copy) {
    let a = stack.pop();
    let result = op(&a);
    match result {
        Ok(result) => stack.push(result),
        Err(e) => panic!("Error: {:?} {:?}", e, a),
    }
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
// pub const OP_NEW_LIST: u16 = 40;
