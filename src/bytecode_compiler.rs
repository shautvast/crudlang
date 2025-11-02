use crate::ast_compiler::{Expression, Function, Statement};
use crate::chunk::Chunk;
use crate::errors::CompilerError;
use crate::tokens::TokenType;
use crate::value::Value;
use crate::vm::{
    OP_ADD, OP_AND, OP_ASSIGN, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CALL, OP_CONSTANT, OP_DEF_BOOL,
    OP_DEF_CHAR, OP_DEF_DATE, OP_DEF_F32, OP_DEF_F64, OP_DEF_I32, OP_DEF_I64, OP_DEF_LIST,
    OP_DEF_MAP, OP_DEF_STRING, OP_DEF_STRUCT, OP_DEF_U32, OP_DEFINE, OP_DIVIDE, OP_EQUAL, OP_GET,
    OP_GREATER, OP_GREATER_EQUAL, OP_LESS, OP_LESS_EQUAL, OP_MULTIPLY, OP_NEGATE, OP_NOT, OP_OR,
    OP_PRINT, OP_RETURN, OP_SHL, OP_SHR, OP_SUBTRACT,
};
use std::collections::HashMap;

pub fn compile(
    namespace: Option<&str>,
    ast: &Vec<Statement>,
    registry: &mut HashMap<String, Chunk>,
) -> Result<(), CompilerError> {
    compile_name(ast, namespace, registry)
}

pub(crate) fn compile_function(
    function: &Function,
    registry: &mut HashMap<String, Chunk>,
    namespace: &str,
) -> Result<Chunk, CompilerError> {
    let mut compiler = Compiler::new(&function.name.lexeme);
    for parm in &function.parameters {
        let name = parm.name.lexeme.clone();
        let var_index = compiler.chunk.add_var(&parm.var_type, &parm.name.lexeme);

        compiler.vars.insert(name, var_index);
        // compiler.emit_bytes(OP_DEFINE, name_index as u16);
    }

    Ok(compiler.compile(&function.body, registry, namespace)?)
}

pub(crate) fn compile_name(
    ast: &Vec<Statement>,
    namespace: Option<&str>,
    registry: &mut HashMap<String, Chunk>,
) -> Result<(), CompilerError> {
    let name = namespace.unwrap_or("main");
    let compiler = Compiler::new(name);
    let chunk = compiler.compile(ast, registry, name)?;
    let qname = if let Some(namespace) = namespace {
        format!("{}.{}", namespace, "main")
    } else {
        "main".to_string()
    };
    registry.insert(qname, chunk);
    Ok(())
}

struct Compiler {
    chunk: Chunk,
    _had_error: bool,
    current_line: usize,
    vars: HashMap<String, usize>,
}

impl Compiler {
    fn new(name: &str) -> Self {
        Self {
            chunk: Chunk::new(name),
            _had_error: false,
            current_line: 0,
            vars: HashMap::new(),
        }
    }

    fn compile(
        mut self,
        ast: &Vec<Statement>,
        registry: &mut HashMap<String, Chunk>,
        namespace: &str,
    ) -> Result<Chunk, CompilerError> {
        for statement in ast {
            self.compile_statement(statement, registry, namespace)?;
        }

        self.emit_byte(OP_RETURN);
        Ok(self.chunk)
    }

    fn compile_statement(
        &mut self,
        statement: &Statement,
        registry: &mut HashMap<String, Chunk>,
        namespace: &str,
    ) -> Result<(), CompilerError> {
        self.current_line = statement.line();
        match statement {
            Statement::VarStmt {
                name,
                var_type,
                initializer,
            } => {
                let name_index = self.chunk.add_var(var_type, &name.lexeme);
                self.vars.insert(name.lexeme.clone(), name_index);
                self.compile_expression(namespace, initializer, registry)?;
                self.emit_bytes(OP_ASSIGN, name_index as u16);
            }
            Statement::PrintStmt { value } => {
                self.compile_expression(namespace, value, registry)?;
                self.emit_byte(OP_PRINT);
            }
            Statement::ExpressionStmt { expression } => {
                self.compile_expression(namespace, expression, registry)?;
            }
            Statement::FunctionStmt { function } => {
                let function_name = function.name.lexeme.clone();
                // self.emit_constant(Value::String(function_name.clone()));
                let compiled_function = compile_function(function, registry, namespace)?;
                registry.insert(
                    format!("{}.{}", self.chunk.name, function_name),
                    compiled_function,
                );
            }
            Statement::ObjectStmt { name, fields } => {
                self.chunk.add_object_def(&name.lexeme, fields);
            }
        }
        Ok(())
    }

    fn compile_expression(
        &mut self,
        namespace: &str,
        expression: &Expression,
        registry: &mut HashMap<String, Chunk>,
    ) -> Result<(), CompilerError> {
        match expression {
            Expression::FunctionCall {
                name, arguments, ..
            } => {
                let qname = format!("{}.{}", namespace, name);
                let name_index = self
                    .chunk
                    .find_constant(&qname)
                    .unwrap_or_else(|| self.emit_constant(qname.into()) as usize);

                for argument in arguments {
                    self.compile_expression(namespace, argument, registry)?;
                }
                self.emit_bytes(OP_CALL, name_index as u16);
                self.emit_byte(arguments.len() as u16);
            }
            Expression::Variable { name, .. } => {
                let name_index = self.vars.get(name).unwrap();
                self.emit_bytes(OP_GET, *name_index as u16);
            }
            Expression::Literal { value, .. } => {
                self.emit_constant(value.clone());
            }
            Expression::List { values, .. } => {
                for expr in values {
                    self.compile_expression(namespace, expr, registry)?;
                }
                self.emit_bytes(OP_DEF_LIST, values.len() as u16);
            }
            Expression::Map { entries, .. } => {
                for (key, value) in entries {
                    self.compile_expression(namespace, key, registry)?;
                    self.compile_expression(namespace, value, registry)?;
                }
                self.emit_bytes(OP_DEF_MAP, entries.len() as u16);
            }
            Expression::Grouping { expression, .. } => {
                self.compile_expression(namespace, expression, registry)?
            }
            Expression::Unary {
                operator, right, ..
            } => {
                self.compile_expression(namespace, right, registry)?;
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
                self.compile_expression(namespace, left, registry)?;
                self.compile_expression(namespace, right, registry)?;
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

    fn emit_byte(&mut self, byte: u16) {
        self.chunk.add(byte, self.current_line);
    }

    fn emit_bytes(&mut self, b1: u16, b2: u16) {
        self.emit_byte(b1);
        self.emit_byte(b2);
    }

    fn emit_constant(&mut self, value: Value) -> u16 {
        let index = self.chunk.add_constant(value) as u16;
        self.emit_bytes(OP_CONSTANT, index);
        index
    }
}
