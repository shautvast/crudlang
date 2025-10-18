use crate::chunk::Chunk;
use crate::opcode::{
    OP_ADD, OP_CONSTANT, OP_DIVIDE, OP_FALSE, OP_MULTIPLY, OP_NEGATE, OP_RETURN, OP_SUBTRACT,
    OP_TRUE,
};
use crate::value::Value;

pub mod chunk;
pub mod compiler;
mod keywords;
pub mod opcode;
pub mod scanner;
mod tokens;
mod value;

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
            print!("[");
            for value in self.stack.iter() {
                print!("{:?} ", value);
            }
            println!("]");
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
                OP_ADD => binary_op(self, add),
                OP_SUBTRACT => binary_op(self, sub),
                OP_MULTIPLY => binary_op(self, mul),
                OP_DIVIDE => binary_op(self, div),
                OP_NEGATE => {
                    let value = &self.pop();
                    let result = -value;
                    match result {
                        Ok(result) => self.push(result),
                        Err(e) => panic!("Error: {:?} {:?}", e, value),
                    }
                }
                OP_RETURN => {
                    println!("return {:?}", self.pop());
                    return Result::Ok;
                }
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
        self.stack.pop().unwrap() //?
    }
}

fn binary_op(stack: &mut Vm, op: impl Fn(&Value, &Value) -> anyhow::Result<Value> + Copy) {
    let a = stack.pop();
    let b = stack.pop();
    let result = op(&a, &b);
    match result {
        Ok(result) => stack.push(result),
        Err(e) => panic!("Error: {:?} {:?} and {:?}", e, a, b),
    }
}

fn add(a: &Value, b: &Value) -> anyhow::Result<Value> {
    a + b
}
fn sub(a: &Value, b: &Value) -> anyhow::Result<Value> {
    a - b
}
fn mul(a: &Value, b: &Value) -> anyhow::Result<Value> {
    a * b
}
fn div(a: &Value, b: &Value) -> anyhow::Result<Value> {
    a / b
}

#[derive(Debug, PartialEq)]
pub enum Result {
    Ok,
    CompileError,
    Error,
}
