use crate::errors::CompilerError::{self, Expected, IncompatibleTypes, ParseError, TooManyParameters, TypeError, UnexpectedIndent, UninitializedVariable};
use crate::tokens::TokenType::{
    Bang, Bool, Char, Colon, Date, Eof, Eol, Equal, False, FloatingPoint, Fn, Greater, GreaterEqual, GreaterGreater,
    Identifier, Indent, Integer, LeftBrace, LeftBracket, LeftParen, Less, LessEqual, LessLess,
    Let, ListType, MapType, Minus, Object, Plus, Print, RightBrace, RightBracket, RightParen, SignedInteger,
    SingleRightArrow, Slash, Star, StringType, True, UnsignedInteger, F32, F64,
    I32, I64, U32, U64,
};
use crate::tokens::{Token, TokenType};
use crate::value::Value;
use log::debug;
use std::collections::HashMap;

pub fn compile(tokens: Vec<Token>) -> Result<Vec<Statement>, CompilerError> {
    let mut compiler = AstCompiler::new(tokens);
    compiler.compile_tokens()
}

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) name: Token,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) return_type: TokenType,
    pub(crate) body: Vec<Statement>,
}

struct AstCompiler {
    tokens: Vec<Token>,
    current: usize,
    had_error: bool,
    vars: Vec<Expression>,
    indent: Vec<usize>,
    functions: HashMap<String, Function>,
}

impl AstCompiler {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            had_error: false,
            vars: vec![],
            indent: vec![0],
            functions: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        self.current = 0;
    }

    fn compile_tokens(&mut self) -> Result<Vec<Statement>,CompilerError> {
        self.collect_functions()?;
        self.reset();
        self.compile()
    }

    fn compile(&mut self) -> Result<Vec<Statement>, CompilerError> {
        if !self.had_error {
            let mut statements = vec![];
            while !self.is_at_end() {
                let statement = self.indent()?;
                if let Some(statement) = statement {
                    statements.push(statement);
                } else {
                    break;
                }
            }
            Ok(statements)
        } else {
            Err(CompilerError::Failure)
        }
    }

    fn collect_functions(&mut self) -> Result<(), CompilerError> {
        while !self.is_at_end() {
            if self.match_token(vec![Fn]) {
                let name_token = self.consume(Identifier, Expected("function name."))?;
                self.consume(LeftParen, Expected("'(' after function name."))?;
                let mut parameters = vec![];
                while !self.check(RightParen) {
                    if parameters.len() >= 25 {
                        return Err(TooManyParameters);
                    }
                    let parm_name = self.consume(Identifier, Expected("a parameter name."))?;

                    self.consume(Colon, Expected(": after parameter name"))?;
                    let var_type = self.peek().token_type;
                    self.vars.push(Expression::Variable {
                        name: parm_name.lexeme.to_string(),
                        var_type,
                        line: parm_name.line,
                    });
                    self.advance();
                    parameters.push(Parameter {
                        name: parm_name,
                        var_type,
                    });
                    if self.peek().token_type == TokenType::Comma {
                        self.advance();
                    }
                }
                self.consume(RightParen, Expected(" ')' after parameters."))?;
                let return_type = if self.check(SingleRightArrow) {
                    self.consume(SingleRightArrow, Expected("->"))?;
                    self.advance().token_type
                } else {
                    TokenType::Void
                };
                self.consume(Colon, Expected("colon (:) after function declaration."))?;
                self.consume(Eol, Expected("end of line."))?;

                let function = Function {
                    name: name_token.clone(),
                    parameters,
                    return_type,
                    body: vec![],
                };

                self.functions.insert(name_token.lexeme, function);
            } else {
                self.advance();
            }
        }
        Ok(())
    }

    fn indent(&mut self) -> Result<Option<Statement>, CompilerError> {
        let expected_indent = *self.indent.last().unwrap();
        // skip empty lines
        while self.check(Eol) {
            self.advance();
        }

        let mut indent_on_line = 0;
        // keep track of indent level
        while self.match_token(vec![Indent]) {
            indent_on_line += 1;
        }
        if indent_on_line > expected_indent {
            Err(UnexpectedIndent(
                indent_on_line, expected_indent
            ))
        } else if indent_on_line < expected_indent {
            self.indent.pop();
            return Ok(None);
        } else {
            Ok(Some(self.declaration()?))
        }
    }

    fn declaration(&mut self) -> Result<Statement, CompilerError> {
        if self.match_token(vec![Fn]) {
            self.function_declaration()
        } else if self.match_token(vec![Let]) {
            self.let_declaration()
        } else if self.match_token(vec![Object]) {
            self.object_declaration()
        } else {
            self.statement()
        }
    }

    fn object_declaration(&mut self) -> Result<Statement, CompilerError> {
        let type_name = self.consume(Identifier, Expected("object name."))?;
        self.consume(Colon, Expected("':' after object name."))?;
        self.consume(Eol, Expected("end of line."))?;

        let mut fields = vec![];

        let expected_indent = self.indent.last().unwrap() + 1;
        // self.indent.push(expected_indent);
        let mut done = false;
        while !done && !self.match_token(vec![Eof]) {
            for _ in 0..expected_indent {
                if self.peek().token_type == Indent {
                    self.advance();
                } else {
                    done = true;
                }
            }
            if !done {
                let field_name = self.consume(Identifier, Expected("an object field name."))?;
                self.consume(Colon, Expected("':' after field name."))?;
                let field_type = self.peek().token_type;
                if field_type.is_type() {
                    self.advance();
                } else {
                    Err(Expected("a type"))?
                }
                fields.push(Parameter {
                    name: field_name,
                    var_type: field_type,
                });
            }
        }
        self.consume(Eol, Expected("end of line."))?;
        Ok(Statement::ObjectStmt {
            name: type_name,
            fields,
        })
    }

    fn function_declaration(&mut self) -> Result<Statement, CompilerError> {
        let name_token = self.consume(Identifier, Expected("function name."))?;
        self.consume(LeftParen, Expected("'(' after function name."))?;
        while !self.check(RightParen) {
            self.advance();
        }

        self.consume(RightParen, Expected("')' after parameters."))?;
        while !self.check(Colon) {
            self.advance();
        }
        self.consume(Colon, Expected("colon (:) after function declaration."))?;
        self.consume(Eol, Expected("end of line."))?;

        let current_indent = self.indent.last().unwrap();
        self.indent.push(current_indent + 1);
        let body = self.compile()?;

        self.functions.get_mut(&name_token.lexeme).unwrap().body = body;

        let function_stmt = Statement::FunctionStmt {
            function: self.functions.get(&name_token.lexeme).unwrap().clone(),
        };
        Ok(function_stmt)
    }

    fn let_declaration(&mut self) -> Result<Statement, CompilerError> {
        let name_token = self.consume(Identifier, Expected("variable name."))?;

        let declared_type = if self.check(Colon) {
            self.advance();
            Some(self.advance().token_type)
        } else {
            None
        };

        if self.match_token(vec![Equal]) {
            let initializer = self.expression()?;
            self.consume(Eol, Expected("end of line after initializer."))?;

            let inferred_type = initializer.infer_type();
            let var_type = match calculate_type(declared_type, inferred_type) {
                Ok(var_type) => var_type,
                Err(e) => {
                    self.had_error = true;
                    return Err(TypeError(name_token.line, Box::new(e)));
                }
            };
            self.vars.push(Expression::Variable {
                name: name_token.lexeme.to_string(),
                var_type,
                line: name_token.line,
            });
            Ok(Statement::VarStmt {
                name: name_token,
                var_type,
                initializer,
            })
        } else {
            Err(UninitializedVariable)?
        }
    }

    fn statement(&mut self) -> Result<Statement, CompilerError> {
        if self.match_token(vec![Print]) {
            self.print_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Statement, CompilerError> {
        let expr = self.expression()?;
        self.consume(Eol, Expected("end of line after print statement."))?;
        Ok(Statement::PrintStmt { value: expr })
    }

    fn expr_statement(&mut self) -> Result<Statement, CompilerError> {
        let expr = self.expression()?;
        self.consume(Eol, Expected("end of line after expression."))?;
        Ok(Statement::ExpressionStmt { expression: expr })
    }

    fn expression(&mut self) -> Result<Expression, CompilerError> {
        self.or()
    }

    fn or(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.and()?;
        self.binary(vec![TokenType::LogicalOr], expr)
    }

    fn and(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.bit_and()?;
        self.binary(vec![TokenType::LogicalAnd], expr)
    }

    fn bit_and(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.bit_or()?;
        self.binary(vec![TokenType::BitAnd], expr)
    }

    fn bit_or(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.bit_xor()?;
        self.binary(vec![TokenType::BitOr], expr)
    }

    fn bit_xor(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.equality()?;
        self.binary(vec![TokenType::BitXor], expr)
    }

    fn equality(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.comparison()?;
        self.binary(vec![TokenType::EqualEqual, TokenType::BangEqual], expr)
    }

    fn comparison(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.bitshift()?;
        self.binary(vec![Greater, GreaterEqual, Less, LessEqual], expr)
    }

    fn bitshift(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.term()?;
        self.binary(vec![GreaterGreater, LessLess], expr)
    }

    fn term(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.factor()?;
        self.binary(vec![Minus, Plus], expr)
    }

    fn factor(&mut self) -> Result<Expression, CompilerError> {
        let expr = self.unary()?;
        self.binary(vec![Slash, Star], expr)
    }

    fn binary(
        &mut self,
        types: Vec<TokenType>,
        mut expr: Expression,
    ) -> Result<Expression, CompilerError> {
        while self.match_token(types.clone()) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expression::Binary {
                line: operator.line,
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, CompilerError> {
        if self.match_token(vec![Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expression::Unary {
                line: self.peek().line,
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expression, CompilerError> {
        debug!("primary {:?}", self.peek());
        Ok(if self.match_token(vec![LeftBracket]) {
            self.list()?
        } else if self.match_token(vec![LeftBrace]) {
            self.map()?
        } else if self.match_token(vec![False]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Bool,
                value: Value::Bool(false),
            }
        } else if self.match_token(vec![True]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Bool,
                value: Value::Bool(true),
            } //, FloatingPoint, Text
        } else if self.match_token(vec![Integer]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Integer,
                value: Value::I64(self.previous().lexeme.parse().map_err(|e|ParseError(format!("{:?}",e)))?),
            }
        } else if self.match_token(vec![FloatingPoint]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: FloatingPoint,
                value: Value::F64(self.previous().lexeme.parse().map_err(|e|ParseError(format!("{:?}",e)))?),
            }
        } else if self.match_token(vec![StringType]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: StringType,
                value: Value::String(self.previous().lexeme.to_string()),
            }
        } else if self.match_token(vec![Char]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Char,
                value: Value::Char(self.previous().lexeme.chars().next().unwrap()),
            }
        } else if self.match_token(vec![LeftParen]) {
            let expr = self.expression()?;
            self.consume(RightParen, Expected("')' after expression."))?;
            Expression::Grouping {
                line: self.peek().line,
                expression: Box::new(expr),
            }
        } else {
            let token = self.advance().clone();
            debug!("{:?}", token);
            if self.match_token(vec![LeftParen]) {
                self.function_call(token.lexeme)?
            } else {
                self.variable_lookup(&token)?
            }
        })
    }

    fn list(&mut self) -> Result<Expression, CompilerError> {
        let mut list = vec![];
        while !self.match_token(vec![RightBracket]) {
            list.push(self.expression()?);
            if self.peek().token_type == TokenType::Comma {
                self.advance();
            } else {
                self.consume(RightBracket, Expected("']' at the end of the list."))?;
                break;
            }
        }
        Ok(Expression::List {
            values: list,
            literaltype: ListType,
            line: self.peek().line,
        })
    }

    fn map(&mut self) -> Result<Expression, CompilerError> {
        let mut entries = vec![];
        while !self.match_token(vec![RightBrace]) {
            let key = self.expression()?;
            self.consume(Colon, Expected("':' after map key."))?;
            let value = self.expression()?;
            entries.push((key, value));
            if self.peek().token_type == TokenType::Comma {
                self.advance();
            } else {
                self.consume(RightBrace, Expected("'}' after map."))?;
                break;
            }
        }
        Ok(Expression::Map {
            entries,
            literaltype: MapType,
            line: self.peek().line,
        })
    }

    fn variable_lookup(&mut self, token: &Token) -> Result<Expression, CompilerError> {
        let (var_name, var_type) = self
            .vars
            .iter()
            .filter_map(|e| {
                if let Expression::Variable { name, var_type, .. } = e {
                    Some((name, var_type))
                } else {
                    None
                }
            })
            .find(|e| e.0 == &token.lexeme)
            .ok_or_else(|| return CompilerError::UndeclaredVariable(token.clone()))?;
        Ok(Expression::Variable {
            name: var_name.to_string(),
            var_type: var_type.clone(),
            line: token.line,
        })
    }

    fn function_call(&mut self, name: String) -> Result<Expression, CompilerError> {
        let function_name = self.functions.get(&name).unwrap().name.lexeme.clone();
        let function = self.functions.get(&function_name).unwrap().clone();

        let mut arguments = vec![];
        while !self.match_token(vec![RightParen]) {
            if arguments.len() >= 25 {
                return Err(TooManyParameters);
            }
            let arg = self.expression()?;
            let arg_type = arg.infer_type();
            if arg_type != function.parameters[arguments.len()].var_type {
                return Err(IncompatibleTypes(
                    function.parameters[arguments.len()].var_type,
                    arg_type
                ));
            }
            arguments.push(arg);
            if self.peek().token_type == TokenType::Comma {
                self.advance();
            } else {
                self.consume(RightParen, Expected("')' after arguments."))?;
                break;
            }
        }
        let return_type = self.functions.get(&name).unwrap().return_type;
        Ok(Expression::FunctionCall {
            line: self.peek().line,
            name,
            arguments,
            return_type,
        })
    }

    fn consume(&mut self, token_type: TokenType, message: CompilerError) -> Result<Token, CompilerError> {
        if self.check(token_type) {
            self.advance();
        } else {
            self.had_error = true;
            // return Err(anyhow::anyhow!(
            //     "{} at {:?}",
            //     message.to_string(),
            //     self.peek()
            // ));
            return Err(message);
        }
        Ok(self.previous().clone())
    }

    fn match_token(&mut self, tokens: Vec<TokenType>) -> bool {
        for tt in tokens {
            if self.check(tt) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }
}

fn calculate_type(
    declared_type: Option<TokenType>,
    inferred_type: TokenType,
) -> Result<TokenType, CompilerError> {
    Ok(if let Some(declared_type) = declared_type {
        if declared_type != inferred_type {
            match (declared_type, inferred_type) {
                (I32, I64) => I32, //need this?
                (I32, Integer) => I32,
                (U32, I64) => U32,
                (U32, Integer) => U32,
                (F32, F64) => F32,
                (F32, FloatingPoint) => F32,
                (F64, I64) => F64,
                (F64, FloatingPoint) => F64,
                (U64, I64) => U64,
                (U64, I32) => U64,
                (StringType, _) => StringType, // meh, this all needs rigorous testing. Update: this is in progress
                _ => {
                    return Err(IncompatibleTypes(
                        declared_type,
                        inferred_type
                    ));
                }
            }
        } else {
            declared_type
        }
    } else {
        match inferred_type {
            Integer | I64 => I64,
            FloatingPoint => F64,
            Bool => Bool,
            Date => Date,
            ListType => ListType,
            MapType => MapType,
            Object => Object,
            _ => return Err(CompilerError::UnexpectedType(inferred_type)),
        }
    })
}

#[derive(Debug, Clone)]
pub enum Statement {
    ExpressionStmt {
        expression: Expression,
    },
    VarStmt {
        name: Token,
        var_type: TokenType,
        initializer: Expression,
    },
    PrintStmt {
        value: Expression,
    },
    FunctionStmt {
        function: Function,
    },
    ObjectStmt {
        name: Token,
        fields: Vec<Parameter>,
    },
}

impl Statement {
    pub fn line(&self) -> usize {
        match self {
            Statement::ExpressionStmt { expression } => expression.line(),
            Statement::VarStmt { name, .. } => name.line,
            Statement::PrintStmt { value } => value.line(),
            Statement::FunctionStmt { function, .. } => function.name.line,
            Statement::ObjectStmt { name, .. } => name.line,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub(crate) name: Token,
    pub(crate) var_type: TokenType,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary {
        line: usize,
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Unary {
        line: usize,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping {
        line: usize,
        expression: Box<Expression>,
    },
    Literal {
        line: usize,
        literaltype: TokenType,
        value: Value,
    },
    List {
        line: usize,
        literaltype: TokenType,
        values: Vec<Expression>,
    },
    Map {
        line: usize,
        literaltype: TokenType,
        entries: Vec<(Expression, Expression)>,
    },
    Variable {
        line: usize,
        name: String,
        var_type: TokenType,
    },
    FunctionCall {
        line: usize,
        name: String,
        arguments: Vec<Expression>,
        return_type: TokenType,
    },
}

impl Expression {
    pub fn line(&self) -> usize {
        match self {
            Self::Binary { line, .. } => *line,
            Self::Unary { line, .. } => *line,
            Self::Grouping { line, .. } => *line,
            Self::Literal { line, .. } => *line,
            Self::List { line, .. } => *line,
            Self::Map { line, .. } => *line,
            Self::Variable { line, .. } => *line,
            Self::FunctionCall { line, .. } => *line,
        }
    }

    pub fn infer_type(&self) -> TokenType {
        match self {
            Self::Binary {
                left,
                operator,
                right,
                ..
            } => {
                let left_type = left.infer_type();
                let right_type = right.infer_type();
                if vec![Greater, Less, GreaterEqual, LessEqual].contains(&operator.token_type) {
                    Bool
                } else if left_type == right_type {
                    // map to determined numeric type if yet undetermined (32 or 64 bits)
                    match left_type {
                        FloatingPoint => F64,
                        Integer => I64,
                        _ => left_type,
                    }
                } else {
                    if let Plus = operator.token_type {
                        // includes string concatenation with numbers
                        // followed by type coercion to 64 bits for numeric types
                        debug!("coerce {} : {}", left_type, right_type);
                        match (left_type, right_type) {
                            (_, StringType) => StringType,
                            (StringType, _) => StringType,
                            (FloatingPoint, _) => F64,
                            (Integer, FloatingPoint) => F64,
                            (Integer, _) => I64,
                            (I64, Integer) => I64,
                            (F64, _) => F64,
                            (U64, U32) => U64,
                            (I64, I32) => I64,
                            // could add a date and a duration. future work
                            // could add a List and a value. also future work
                            // could add a Map and a tuple. Will I add tuple types? Future work!
                            _ => panic!("Unexpected coercion"),
                        }
                        // could have done some fall through here, but this will fail less gracefully,
                        // so if my thinking is wrong or incomplete it will panic
                    } else {
                        // type coercion to 64 bits for numeric types
                        debug!("coerce {} : {}", left_type, right_type);
                        match (left_type, right_type) {
                            (FloatingPoint, _) => F64,
                            (Integer, FloatingPoint) => F64,
                            (Integer, I64) => I64,
                            (I64, FloatingPoint) => F64,
                            (F64, _) => F64,
                            (U64, U32) => U64,
                            (I64, I32) => I64,
                            (I64, Integer) => I64,
                            _ => panic!("Unexpected coercion"),
                        }
                    }
                }
            }
            Self::Grouping { expression, .. } => expression.infer_type(),
            Self::Literal { literaltype, .. } => literaltype.clone(),
            Self::List { literaltype, .. } => literaltype.clone(),
            Self::Map { literaltype, .. } => literaltype.clone(),
            Self::Unary {
                right, operator, ..
            } => {
                let literal_type = right.infer_type();
                if literal_type == Integer && operator.token_type == Minus {
                    SignedInteger
                } else {
                    UnsignedInteger
                }
            }
            Self::Variable { var_type, .. } => var_type.clone(),
            Self::FunctionCall { return_type, .. } => return_type.clone(),
        }
    }
}
