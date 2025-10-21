use crate::chunk::Chunk;
use crate::value::Value;
use anyhow::anyhow;
use std::collections::HashMap;
use tracing::debug;

pub fn interpret(chunk: Chunk) -> anyhow::Result<Value> {
    let mut vm = Vm {
        chunk,
        ip: 0,
        stack: vec![],
        local_vars: HashMap::new(),
    };
    vm.run()
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    local_vars: HashMap<String, Value>,
}

impl Vm {
    fn run(&mut self) -> anyhow::Result<Value> {
        loop {
            debug!("{:?}", self.stack);
            let opcode = self.chunk.code[self.ip];
            self.ip += 1;
            match opcode {
                OP_CONSTANT | OP_FALSE | OP_TRUE => {
                    let value = &self.chunk.constants[self.chunk.code[self.ip] as usize];
                    self.ip += 1;
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
                    let name = self.read_constant();
                    let value = self.pop();
                    self.local_vars.insert(name, value);
                }
                OP_GET => {
                    let name = self.read_constant();
                    let value = self.local_vars.get(&name).unwrap();
                    self.push(value.clone()); // not happy
                    debug!("after get {:?}", self.stack);
                }
                _ => {}
            }
        }
    }

    fn read_constant(&mut self) -> String {
        let name = self.chunk.constants[self.chunk.code[self.ip] as usize].to_string();
        self.ip += 1;
        name
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
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
        Err(e) => println!("Error: {} {:?} and {:?}", e.to_string(), a, b),
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
pub const OP_TRUE: u16 = 9;
pub const OP_FALSE: u16 = 10;
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
pub const OP_DEFINE: u16 = 26;
pub const OP_GET: u16 = 27;
