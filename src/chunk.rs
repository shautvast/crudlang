use crate::ast_compiler::Parameter;
use crate::tokens::TokenType;
use crate::value::Value;
use crate::vm::{
    OP_ADD, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CALL, OP_CONSTANT, OP_DEF_BOOL, OP_DEF_F32,
    OP_DEF_F64, OP_DEF_I32, OP_DEF_I64, OP_DEF_LIST, OP_DEF_MAP, OP_DEF_STRING, OP_DEFINE,
    OP_DIVIDE, OP_EQUAL, OP_GET, OP_GREATER, OP_GREATER_EQUAL, OP_LESS, OP_LESS_EQUAL, OP_MULTIPLY,
    OP_NEGATE, OP_NOT, OP_POP, OP_PRINT, OP_RETURN, OP_SHL, OP_SHR, OP_SUBTRACT,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) name: String,
    pub code: Vec<u16>,
    pub constants: Vec<Value>,
    lines: Vec<usize>,
    pub(crate) object_defs: HashMap<String, Vec<Parameter>>,
    pub(crate) function_parameters: Vec<Parameter>,
    pub vars: Vec<(TokenType, String)>,
}

impl Chunk {
    pub(crate) fn find_constant(&self, p0: &String) -> Option<usize> {
        for (i, constant) in self.constants.iter().enumerate() {
            if let Value::String(s) = constant
                && s == p0
            {
                return Some(i);
            }
        }
        None
    }
}

impl Chunk {
    pub(crate) fn new(name: &str) -> Chunk {
        Chunk {
            name: name.to_string(),
            code: Vec::new(),
            constants: vec![],
            lines: vec![],
            object_defs: HashMap::new(),
            function_parameters: vec![],
            vars: vec![],
        }
    }

    pub(crate) fn add(&mut self, byte: u16, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub(crate) fn add_constant(&mut self, value: impl Into<Value>) -> usize {
        self.constants.push(value.into());
        self.constants.len() - 1
    }

    pub(crate) fn add_var(&mut self, var_type: &TokenType, name: &str) -> usize {
        self.vars.push((var_type.clone(), name.to_string()));
        self.vars.len() - 1
    }

    pub(crate) fn add_object_def(&mut self, name: &str, fields: &[Parameter]) {
        self.object_defs.insert(name.to_string(), fields.to_vec());
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
            OP_CONSTANT => self.constant_inst("LDC", offset),
            OP_ADD => self.simple_inst("ADD", offset),
            OP_SUBTRACT => self.simple_inst("SUB", offset),
            OP_MULTIPLY => self.simple_inst("MUL", offset),
            OP_DIVIDE => self.simple_inst("DIV", offset),
            OP_BITAND => self.simple_inst("BITAND", offset),
            OP_BITOR => self.simple_inst("BITOR", offset),
            OP_BITXOR => self.simple_inst("BITXOR", offset),
            OP_NEGATE => self.simple_inst("NEG", offset),
            OP_NOT => self.simple_inst("NOT", offset),
            OP_RETURN => self.simple_inst("RET", offset),
            OP_SHL => self.simple_inst("SHL", offset),
            OP_SHR => self.simple_inst("SHR", offset),
            OP_LESS => self.simple_inst("LT", offset),
            OP_LESS_EQUAL => self.simple_inst("LTE", offset),
            OP_GREATER => self.simple_inst("GT", offset),
            OP_GREATER_EQUAL => self.simple_inst("GTE", offset),
            OP_EQUAL => self.simple_inst("EQ", offset),
            OP_PRINT => self.simple_inst("PRT", offset),
            OP_POP => self.simple_inst("POP", offset),
            OP_DEFINE => self.constant_inst("DEF", offset),
            OP_DEF_STRING => self.constant_inst("DEFSTR", offset),
            OP_DEF_I32 => self.constant_inst("DEFI32", offset),
            OP_DEF_I64 => self.constant_inst("DEFI64", offset),
            OP_DEF_F32 => self.constant_inst("DEFF32", offset),
            OP_DEF_F64 => self.constant_inst("DEFF64", offset),
            OP_DEF_BOOL => self.constant_inst("DEFBOOL", offset),
            OP_CALL => self.call_inst("CALL", offset),
            OP_GET => self.constant_inst("GET", offset),
            OP_DEF_LIST => self.new_inst("DEFLIST", offset),
            OP_DEF_MAP => self.new_inst("DEFMAP", offset),
            _ => {
                println!("Unknown instruction {}", instruction);
                offset + 1
            }
        }
    }

    fn simple_inst(&self, op: &str, offset: usize) -> usize {
        println!("{}", op);
        offset + 1
    }

    fn call_inst(&self, op: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        let num_args = self.code[offset + 2];
        println!(
            "{} {}:{}({}):",
            op, constant, &self.constants[constant as usize], num_args
        );
        offset + 3
    }

    fn new_inst(&self, op: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        let len = self.code[offset + 2];
        print!("{} len: {}:", op, len);
        self.print_value(&self.constants[constant as usize]);
        offset + 3
    }

    fn constant_inst(&self, op: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{} {}:", op, constant);
        self.print_value(&self.constants[constant as usize]);
        offset + 2
    }

    fn print_value(&self, value: &Value) {
        println!("{}", value);
    }
}
