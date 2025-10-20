use anyhow::anyhow;
use tracing::debug;
use crate::chunk::Chunk;
use crate::value::Value;

pub fn interpret(chunk: Chunk) -> Result {
    let mut vm = Vm {
        chunk,
        ip: 0,
        stack: vec![],
    };
    vm.run()
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    fn run(&mut self) -> Result {
        loop {
            debug!("[");
            for value in self.stack.iter() {
                debug!("{:?} ", value);
            }
            debug!("]");
            let opcode = self.chunk.code[self.ip];
            self.ip += 1;
            match opcode {
                OP_CONSTANT => {
                    let value = &self.chunk.constants[self.chunk.code[self.ip] as usize];
                    self.ip += 1;
                    self.push(value.clone());
                }
                OP_FALSE => self.push(Value::Bool(false)),
                OP_TRUE => self.push(Value::Bool(true)),
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
                    // println!("{:?}", self.pop());
                    return Result::Ok(self.pop());
                }
                OP_SHL => binary_op(self, |a, b| a << b),
                OP_SHR => binary_op(self, |a, b| a >> b),
                _ => {}
            }
        }
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
    let a = vm.pop();
    let b = vm.pop();

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

#[derive(Debug)]
pub enum Result {
    Ok(Value),
    CompileError,
    Error,
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
