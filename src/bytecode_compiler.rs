use crate::ast_compiler::Expression::NamedParameter;
use crate::ast_compiler::{Expression, Function, Parameter, Statement};
use crate::chunk::Chunk;
use crate::errors::CompilerError::{IncompatibleTypes, UndeclaredVariable};
use crate::errors::{CompilerError, CompilerErrorAtLine};
use crate::symbol_builder::{Symbol, calculate_type, infer_type};
use crate::tokens::TokenType;
use crate::tokens::TokenType::Unknown;
use crate::value::Value;
use crate::vm::{
    OP_ADD, OP_AND, OP_ASSIGN, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CALL, OP_CALL_BUILTIN,
    OP_CONSTANT, OP_DEF_LIST, OP_DEF_MAP, OP_DIVIDE, OP_EQUAL, OP_GET, OP_GREATER,
    OP_GREATER_EQUAL, OP_LESS, OP_LESS_EQUAL, OP_LIST_GET, OP_MULTIPLY, OP_NEGATE, OP_NOT, OP_OR,
    OP_PRINT, OP_RETURN, OP_SHL, OP_SHR, OP_SUBTRACT,
};
use crate::{Registry, SymbolTable};
use std::collections::HashMap;
use std::mem;

pub fn compile(
    qualified_name: Option<&str>,
    ast: &Vec<Statement>,
    symbols: &SymbolTable,
    registry: &mut Registry,
) -> Result<(), CompilerErrorAtLine> {
    compile_in_namespace(ast, qualified_name, symbols, registry)
}

pub(crate) fn compile_function(
    function: &Function,
    symbols: &SymbolTable,
    registry: &mut Registry,
    namespace: &str,
) -> Result<Chunk, CompilerErrorAtLine> {
    let fn_name = &function.name.lexeme;
    let mut compiler = Compiler::new(fn_name);
    for parm in &function.parameters {
        let name = parm.name.lexeme.clone();
        let var_index = compiler.chunk.add_var(&parm.var_type, &parm.name.lexeme);

        compiler.vars.insert(name, var_index);
    }
    let mut chunk = compiler.compile(&function.body, symbols, registry, namespace)?;
    chunk.function_parameters = function.parameters.to_vec();
    Ok(chunk)
}

pub(crate) fn compile_in_namespace(
    ast: &Vec<Statement>,
    namespace: Option<&str>,
    symbols: &SymbolTable,
    registry: &mut Registry,
) -> Result<(), CompilerErrorAtLine> {
    let name = namespace.unwrap_or("main");
    let mut compiler = Compiler::new(name);
    let chunk = compiler.compile(ast, symbols, registry, name)?;
    let qname = if let Some(namespace) = namespace {
        format!("{}/{}", namespace, "main")
    } else {
        "main".to_string()
    };
    registry.insert(qname, chunk);
    Ok(())
}

pub(crate) struct Compiler {
    chunk: Chunk,
    _had_error: bool,
    current_line: usize,
    vars: HashMap<String, usize>,
}

impl Compiler {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            chunk: Chunk::new(name),
            _had_error: false,
            current_line: 0,
            vars: HashMap::new(),
        }
    }

    pub(crate) fn compile(
        &mut self,
        ast: &Vec<Statement>,
        symbols: &SymbolTable,
        registry: &mut Registry,
        namespace: &str,
    ) -> Result<Chunk, CompilerErrorAtLine> {
        for statement in ast {
            self.compile_statement(statement, symbols, registry, namespace)?;
        }

        self.emit_byte(OP_RETURN);
        let chunk = self.chunk.clone();
        self.chunk.code.clear(); // in case the compiler is reused, clear it for the next compilation. This is for the REPL
        Ok(chunk)
    }

    fn raise(&self, error: CompilerError) -> CompilerErrorAtLine {
        CompilerErrorAtLine::raise(error, self.current_line)
    }

    fn compile_statement(
        &mut self,
        statement: &Statement,
        symbols: &SymbolTable,
        registry: &mut Registry,
        namespace: &str,
    ) -> Result<(), CompilerErrorAtLine> {
        self.current_line = statement.line();
        match statement {
            Statement::VarStmt {
                name, initializer, ..
            } => {
                let name = name.lexeme.as_str();
                let var = symbols.get(name);
                if let Some(Symbol::Variable { var_type, .. }) = var {
                    let inferred_type = infer_type(initializer, symbols);
                    let calculated_type =
                        calculate_type(var_type, &inferred_type).map_err(|e| self.raise(e))?;
                    if var_type != &Unknown && var_type != &calculated_type {
                        return Err(
                            self.raise(IncompatibleTypes(var_type.clone(), calculated_type))
                        );
                    }
                    let name_index = self.chunk.add_var(var_type, name);
                    self.vars.insert(name.to_string(), name_index);
                    self.compile_expression(namespace, initializer, symbols, registry)?;
                    self.emit_bytes(OP_ASSIGN, name_index as u16);
                } else {
                    return Err(self.raise(UndeclaredVariable(name.to_string())));
                }
            }
            // replace with function
            Statement::PrintStmt { value } => {
                self.compile_expression(namespace, value, symbols, registry)?;
                self.emit_byte(OP_PRINT);
            }
            Statement::ExpressionStmt { expression } => {
                self.compile_expression(namespace, expression, symbols, registry)?;
            }
            Statement::FunctionStmt { function } => {
                let function_name = function.name.lexeme.clone();
                let compiled_function = compile_function(function, symbols, registry, namespace)?;
                registry.insert(
                    format!("{}/{}", self.chunk.name, function_name),
                    compiled_function,
                );
            }
            Statement::ObjectStmt { name, fields } => {
                self.chunk.add_object_def(&name.lexeme, fields);
            }
            Statement::GuardStatement { .. } => {
                unimplemented!("guard statement")
            }
        }
        Ok(())
    }

    fn compile_expression(
        &mut self,
        namespace: &str,
        expression: &Expression,
        symbols: &SymbolTable,
        registry: &mut Registry,
    ) -> Result<(), CompilerErrorAtLine> {
        match expression {
            Expression::FunctionCall {
                name, arguments, ..
            } => {
                let name_index = self
                    .chunk
                    .find_constant(name)
                    .unwrap_or_else(|| self.chunk.add_constant(Value::String(name.to_string())));
                let function = symbols.get(name);
                match function {
                    Some(Symbol::Function { parameters, .. }) => {
                        self.get_arguments_in_order(
                            namespace, symbols, registry, arguments, parameters,
                        )?;

                        self.emit_bytes(OP_CALL, name_index as u16);
                        self.emit_byte(arguments.len() as u16);
                    }
                    // constructor function
                    Some(Symbol::Object { fields, .. }) => {
                        self.get_arguments_in_order(
                            namespace, symbols, registry, arguments, fields,
                        )?;
                        self.emit_bytes(OP_CALL, name_index as u16);
                        self.emit_byte(arguments.len() as u16);
                    }
                    _ => {
                        return Err(self.raise(CompilerError::FunctionNotFound(name.to_string())));
                    }
                }
            }
            Expression::MethodCall {
                receiver,
                method_name,
                arguments,
                ..
            } => {
                self.compile_expression(namespace, receiver, symbols, registry)?;
                let receiver_type = infer_type(receiver, symbols).to_string();

                let type_index = self
                    .chunk
                    .find_constant(&receiver_type)
                    .unwrap_or_else(|| self.chunk.add_constant(Value::String(receiver_type)));

                let name_index = self.chunk.find_constant(method_name).unwrap_or_else(|| {
                    self.chunk
                        .add_constant(Value::String(method_name.to_string()))
                });
                //TODO lookup parameters for builtin
                self.get_arguments_in_order(namespace, symbols, registry, arguments, &vec![])?;
                self.emit_byte(OP_CALL_BUILTIN);
                self.emit_byte(name_index as u16);
                self.emit_byte(type_index as u16);
                self.emit_byte(arguments.len() as u16);
            }
            Expression::Variable { name, .. } => {
                let name_index = self.vars.get(name);
                if let Some(name_index) = name_index {
                    self.emit_bytes(OP_GET, *name_index as u16);
                } else {
                    return Err(self.raise(UndeclaredVariable(name.to_string())));
                }
            }
            Expression::Literal { value, .. } => {
                self.emit_constant(value.clone());
            }
            Expression::List { values, .. } => {
                for expr in values {
                    self.compile_expression(namespace, expr, symbols, registry)?;
                }
                self.emit_bytes(OP_DEF_LIST, values.len() as u16);
            }
            Expression::Map { entries, .. } => {
                for (key, value) in entries {
                    self.compile_expression(namespace, key, symbols, registry)?;
                    self.compile_expression(namespace, value, symbols, registry)?;
                }
                self.emit_bytes(OP_DEF_MAP, entries.len() as u16);
            }
            Expression::Grouping { expression, .. } => {
                self.compile_expression(namespace, expression, symbols, registry)?
            }
            Expression::Unary {
                operator, right, ..
            } => {
                self.compile_expression(namespace, right, symbols, registry)?;
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
                self.compile_expression(namespace, left, symbols, registry)?;
                self.compile_expression(namespace, right, symbols, registry)?;
                match operator.token_type {
                    TokenType::Plus => self.emit_byte(OP_ADD),
                    TokenType::Minus => self.emit_byte(OP_SUBTRACT),
                    TokenType::Star => self.emit_byte(OP_MULTIPLY),
                    TokenType::Slash => self.emit_byte(OP_DIVIDE),
                    TokenType::BitAnd => self.emit_byte(OP_BITAND),
                    TokenType::Pipe => self.emit_byte(OP_BITOR),
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
            Expression::Stop { .. } => {}
            NamedParameter { value, .. } => {
                self.compile_expression(namespace, value, symbols, registry)?
            }
            Expression::ListGet { index, list } => {
                self.compile_expression(namespace, list, symbols, registry)?;
                self.compile_expression(namespace, index, symbols, registry)?;
                self.emit_byte(OP_LIST_GET);
            }
            Expression::MapGet { .. } => {}
            Expression::FieldGet { .. } => {}
        }
        Ok(())
    }

    // any unnamed parameters must be passed in order
    // named parameters do not have to be passed in order, but they do need to be evaluated in the order of the called function/constructor
    fn get_arguments_in_order(
        &mut self,
        namespace: &str,
        symbols: &SymbolTable,
        registry: &mut Registry,
        arguments: &Vec<Expression>,
        parameters: &Vec<Parameter>,
    ) -> Result<(), CompilerErrorAtLine> {
        for parameter in parameters {
            for argument in arguments {
                if let NamedParameter { name, value, .. } = argument {
                    if name.lexeme == parameter.name.lexeme {
                        let value_type = infer_type(value, symbols);
                        if parameter.var_type != value_type {
                            return Err(self.raise(CompilerError::IncompatibleTypes(
                                parameter.var_type.clone(),
                                value_type,
                            )));
                        } else {
                            self.compile_expression(namespace, argument, symbols, registry)?;
                            break;
                        }
                    }
                } else {
                    self.compile_expression(namespace, argument, symbols, registry)?;
                    break;
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
