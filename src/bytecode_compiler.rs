use crate::ast_compiler::{Expression, Function, Statement};
use crate::chunk::Chunk;
use crate::tokens::TokenType;
use crate::value::Value;
use crate::vm::{
    OP_ADD, OP_AND, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CALL, OP_CONSTANT, OP_DEF_BOOL, OP_DEF_CHAR,
    OP_DEF_DATE, OP_DEF_F32, OP_DEF_F64, OP_DEF_I32, OP_DEF_I64, OP_DEF_LIST,
    OP_DEF_MAP, OP_DEF_STRING, OP_DEF_STRUCT, OP_DEFINE, OP_DIVIDE, OP_EQUAL, OP_GET, OP_GREATER,
    OP_GREATER_EQUAL, OP_LESS, OP_LESS_EQUAL, OP_MULTIPLY, OP_NEGATE, OP_NOT, OP_OR,
    OP_PRINT, OP_RETURN, OP_SHL, OP_SHR, OP_SUBTRACT,
};
use std::collections::HashMap;

pub fn compile(ast: &Vec<Statement>) -> anyhow::Result<Chunk> {
    compile_name(ast, "_")
}

pub(crate) fn compile_function(function: &Function) -> anyhow::Result<Chunk> {
    let mut compiler = Compiler::new(&function.name.lexeme);
    for parm in &function.parameters {
        let name = parm.name.lexeme.clone();
        let name_index = compiler.chunk.add_constant(Value::String(name.clone()));
        compiler.vars.insert(name, name_index);
        compiler.emit_bytes(OP_DEFINE, name_index as u16);
    }

    Ok(compiler.compile(&function.body)?)
}

pub(crate) fn compile_name(ast: &Vec<Statement>, name: &str) -> anyhow::Result<Chunk> {
    let compiler = Compiler::new(name);
    Ok(compiler.compile(ast)?)
}

struct Compiler {
    chunk: Chunk,
    had_error: bool,
    current_line: usize,
    vars: HashMap<String, usize>,
    functions: HashMap<String, usize>,
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self {
            chunk: Chunk::new(name),
            had_error: false,
            current_line: 0,
            vars: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn compile(mut self, ast: &Vec<Statement>) -> anyhow::Result<Chunk> {
        for statement in ast {
            self.compile_statement(statement)?
        }

        self.emit_byte(OP_RETURN);
        Ok(self.chunk)
    }

    fn compile_statement(&mut self, statement: &Statement) -> anyhow::Result<()> {
        self.current_line = statement.line();
        match statement {
            Statement::VarStmt {
                name,
                var_type,
                initializer,
            } => {
                let name_index = self.chunk.add_constant(Value::String(name.lexeme.clone()));
                self.vars.insert(name.lexeme.clone(), name_index);
                self.compile_expression(initializer)?;
                self.define_variable(var_type, name_index)?;
                if let Expression::List {values, .. } = initializer {
                    self.emit_byte(values.len() as u16);
                }
            }
            Statement::PrintStmt { value } => {
                self.compile_expression(value)?;
                self.emit_byte(OP_PRINT);
            }
            Statement::ExpressionStmt { expression } => {
                self.compile_expression(expression)?;
            }
            Statement::FunctionStmt { function } => {
                let function_name = function.name.lexeme.clone();
                let compiled_function = compile_function(function)?;
                let name_index = self.chunk.add_function(compiled_function);
                self.functions.insert(function_name, name_index);
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> anyhow::Result<()> {
        match expression {
            Expression::FunctionCall {
                name, arguments, ..
            } => {
                let function_index = *self.functions.get(name).unwrap();
                for argument in arguments {
                    self.compile_expression(argument)?;
                }
                self.emit_bytes(OP_CALL, function_index as u16);
                self.emit_byte(arguments.len() as u16);
            }
            Expression::Variable { name, .. } => {
                let name_index = self.vars.get(name).unwrap();
                self.emit_bytes(OP_GET, *name_index as u16);
            }
            Expression::Literal { value, .. } => self.emit_constant(value),
            Expression::List { values, .. } => {
                for expr in values {
                    self.compile_expression(expr)?;
                }
                // self.emit_bytes(OP_NEW_LIST, values.len() as u16);
            }
            Expression::Grouping { expression, .. } => self.compile_expression(expression)?,
            Expression::Unary {
                operator, right, ..
            } => {
                self.compile_expression(right)?;
                match operator.token_type {
                    TokenType::Minus => {
                        self.emit_byte(OP_NEGATE);
                    }
                    TokenType::Bang => {
                        self.emit_byte(OP_NOT);
                    }
                    _ => unimplemented!("unary other than ! and -"),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match operator.token_type {
                    TokenType::Plus => self.emit_byte(OP_ADD),
                    TokenType::Minus => self.emit_byte(OP_SUBTRACT),
                    TokenType::Star => self.emit_byte(OP_MULTIPLY),
                    TokenType::Slash => self.emit_byte(OP_DIVIDE),
                    TokenType::BitAnd => self.emit_byte(OP_BITAND),
                    TokenType::BitOr => self.emit_byte(OP_BITOR),
                    TokenType::BitXor => self.emit_byte(OP_BITXOR),
                    TokenType::GreaterGreater => self.emit_byte(OP_SHR),
                    TokenType::LessLess => self.emit_byte(OP_SHL),
                    TokenType::EqualEqual => self.emit_byte(OP_EQUAL),
                    TokenType::Greater => self.emit_byte(OP_GREATER),
                    TokenType::GreaterEqual => self.emit_byte(OP_GREATER_EQUAL),
                    TokenType::Less => self.emit_byte(OP_LESS),
                    TokenType::LessEqual => self.emit_byte(OP_LESS_EQUAL),
                    TokenType::LogicalAnd => self.emit_byte(OP_AND),
                    TokenType::LogicalOr => self.emit_byte(OP_OR),
                    _ => unimplemented!("binary other than plus, minus, star, slash"),
                }
            }
        }
        Ok(())
    }

    fn define_variable(&mut self, var_type: &TokenType, name_index: usize) -> anyhow::Result<()> {
        let def_op = match var_type {
            TokenType::I32 => OP_DEF_I32,
            TokenType::I64 => OP_DEF_I64,
            TokenType::U32 => OP_DEF_I64,
            TokenType::U64 => OP_DEF_I64,
            TokenType::F32 => OP_DEF_F32,
            TokenType::F64 => OP_DEF_F64,
            TokenType::Date => OP_DEF_DATE,
            TokenType::StringType => OP_DEF_STRING,
            TokenType::Char => OP_DEF_CHAR,
            TokenType::Bool => OP_DEF_BOOL,
            TokenType::ListType => OP_DEF_LIST,
            TokenType::MapType => OP_DEF_MAP,
            TokenType::Object => OP_DEF_STRUCT,
            _ => unimplemented!("{}", var_type),
        };

        self.emit_bytes(def_op, name_index as u16);
        Ok(())
    }

    fn emit_byte(&mut self, byte: u16) {
        self.chunk.add(byte, self.current_line);
    }

    fn emit_bytes(&mut self, b1: u16, b2: u16) {
        self.emit_byte(b1);
        self.emit_byte(b2);
    }

    fn emit_constant(&mut self, value: &Value) {
        let index = self.chunk.add_constant(value.clone());
        self.emit_bytes(OP_CONSTANT, index as u16);
    }
}
