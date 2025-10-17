use crate::chunk::Chunk;
use crate::opcode::{OP_ADD,OP_SUBTRACT, OP_MULTIPLY, OP_DIVIDE, OP_CONSTANT, OP_NEGATE, OP_RETURN};
use crate::value::Value;

pub mod chunk;
mod keywords;
pub mod opcode;
pub mod scanner;
mod tokens;
mod value;
pub mod compiler;

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
                OP_ADD => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(binary_op(a, b, |a, b| a + b))
                }
                OP_SUBTRACT => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(binary_op(a, b, |a, b| a - b))
                }
                OP_MULTIPLY => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(binary_op(a, b, |a, b| a * b))
                }
                OP_DIVIDE => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(binary_op(a, b, |a, b| a / b))
                }
                OP_NEGATE => {
                    let value = self.pop();
                    self.push(-value);
                }
                // OP_RETURN => {
                //     println!("{:?}", self.pop());
                //     return Result::Ok;
                // }
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

fn binary_op(a: Value, b: Value, op: impl Fn(Value, Value) -> Value) -> Value {
    op(a, b)
}

#[derive(Debug, PartialEq)]
pub enum Result {
    Ok,
    CompileError,
    Error,
}
