use crate::ast_compiler::{Expression, Statement};
use crate::chunk::Chunk;
use crate::tokens::TokenType;
use crate::value::Value;
use crate::vm::{OP_ADD, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CONSTANT, OP_DEF_BOOL, OP_DEF_CHAR, OP_DEF_DATE, OP_DEF_F64, OP_DEF_I32, OP_DEF_I64, OP_DEF_LIST, OP_DEF_MAP, OP_DEF_STRING, OP_DEF_STRUCT, OP_DEFINE, OP_DIVIDE, OP_EQUAL, OP_GREATER, OP_GREATER_EQUAL, OP_LESS, OP_LESS_EQUAL, OP_MULTIPLY, OP_NEGATE, OP_NOT, OP_RETURN, OP_SHL, OP_SHR, OP_SUBTRACT, OP_DEF_F32, OP_GET, OP_PRINT};

pub fn compile(ast: Vec<Statement>) -> anyhow::Result<Chunk> {
    let compiler = Compiler::new();
    Ok(compiler.compile(ast)?)
}

struct Compiler {
    chunk: Chunk,
    had_error: bool,
    current_line: usize,
}

impl Compiler {
    fn new() -> Self {
        Self {
            chunk: Chunk::new("main"),
            had_error: false,
            current_line: 0,
        }
    }

    fn compile(mut self, ast: Vec<Statement>) -> anyhow::Result<Chunk> {
        for statement in &ast {
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
                let name_index= self.chunk.add_constant(Value::String(name.lexeme.clone()));
                self.compile_expression(initializer)?;
                self.define_variable(var_type, name_index)?
            }
            Statement::Print {value} => {
                self.compile_expression(value)?;
                self.emit_byte(OP_PRINT);
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> anyhow::Result<()> {
        match expression {
            Expression::Literal { value, .. } => self.emit_constant(value),
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
            TokenType::String => OP_DEF_STRING,
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
