use tracing::debug;
use crate::value::Value;
use crate::{
    OP_ADD, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CONSTANT, OP_DIVIDE, OP_FALSE, OP_MULTIPLY,
    OP_NEGATE, OP_RETURN, OP_SUBTRACT, OP_TRUE,
};

pub struct Chunk {
    name: String,
    pub code: Vec<u16>,
    pub constants: Vec<Value>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new(name: &str) -> Chunk {
        Chunk {
            name: name.to_string(),
            code: Vec::new(),
            constants: vec![],
            lines: vec![],
        }
    }

    pub fn add(&mut self, byte: u16, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: impl Into<Value>) -> usize {
        self.constants.push(value.into());
        self.constants.len() - 1
    }

    pub fn disassemble(&self) {
        println!("== {} ==", self.name);
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_inst(offset);
        }
        println!();
    }

    fn disassemble_inst(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:04} ", self.lines[offset]);
        }
        let instruction = self.code[offset];
        match instruction {
            OP_CONSTANT => self.constant_inst("OP_CONSTANT", offset),
            OP_ADD => self.simple_inst("OP_ADD", offset),
            OP_FALSE => self.simple_inst("OP_FALSE", offset),
            OP_TRUE => self.simple_inst("OP_TRUE", offset),
            OP_SUBTRACT => self.simple_inst("OP_SUBTRACT", offset),
            OP_MULTIPLY => self.simple_inst("OP_MULTIPLY", offset),
            OP_DIVIDE => self.simple_inst("OP_DIVIDE", offset),
            OP_BITAND => self.simple_inst("OP_BITAND", offset),
            OP_NEGATE => self.simple_inst("OP_NEGATE", offset),
            OP_RETURN => self.simple_inst("OP_RETURN", offset),
            _ => {
                println!("Unknown instruction");
                offset + 1
            }
        }
    }

    fn simple_inst(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_inst(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        debug!("{} {} ", name, constant);
        self.print_value(&self.constants[constant as usize]);
        offset + 2
    }

    fn print_value(&self, value: &Value) {
        println!("'{:?}'", value);
    }
}
