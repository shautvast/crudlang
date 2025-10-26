use crate::ast_compiler::Expression::Variable;
use crate::tokens::TokenType::{
    Bang, Bool, Char, Colon, Date, Eol, Equal, F32, F64, False, FloatingPoint, Greater,
    GreaterEqual, I32, I64, Identifier, Integer, LeftParen, Less, LessEqual, Let, ListType,
    MapType, Minus, Object, Plus, Print, RightParen, Slash, Star, Text, True, U32, U64,
};
use crate::tokens::{Token, TokenType};
use crate::value::Value;
use log::debug;

pub fn compile(tokens: Vec<Token>) -> anyhow::Result<Vec<Statement>> {
    let mut compiler = AstCompiler::new(tokens);
    compiler.compile()
}

struct AstCompiler {
    tokens: Vec<Token>,
    current: usize,
    had_error: bool,
    vars: Vec<Expression>,
}

impl AstCompiler {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            had_error: false,
            vars: vec![],
        }
    }

    fn compile(&mut self) -> anyhow::Result<Vec<Statement>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration()?)
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> anyhow::Result<Statement> {
        if self.match_token(vec![Let]) {
            self.let_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> anyhow::Result<Statement> {
        let name_token = self.consume(Identifier, "Expect variable name.")?;

        let declared_type = if self.check(Colon) {
            self.advance();
            Some(self.advance().token_type)
        } else {
            None
        };

        if self.match_token(vec![Equal]) {
            let initializer = self.expression()?;
            self.consume(Eol, "Expect end of line after initializer.")?;

            let inferred_type = initializer.infer_type();
            let var_type = match calculate_type(declared_type, inferred_type) {
                Ok(var_type) => var_type,
                Err(e) => {
                    println!("error at line {}", name_token.line);
                    self.had_error = true;
                    return Err(e);
                }
            };
            self.vars.push(Variable {
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
            Err(anyhow::anyhow!("Uninitialized variables are not allowed."))?
        }
    }

    fn statement(&mut self) -> anyhow::Result<Statement> {
        if self.match_token(vec![Print]) {
            self.print_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> anyhow::Result<Statement> {
        let expr = self.expression()?;
        self.consume(Eol, "Expect end of line after expression.")?;
        Ok(Statement::Print { value: expr })
    }

    fn expr_statement(&mut self) -> anyhow::Result<Statement> {
        let expr = self.expression()?;
        self.consume(Eol, "Expect end of line after expression.")?;
        Ok(Statement::ExpressionStmt { expression: expr })
    }

    fn expression(&mut self) -> anyhow::Result<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> anyhow::Result<Expression> {
        let mut expr = self.comparison()?;
        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
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

    fn comparison(&mut self) -> anyhow::Result<Expression> {
        let mut expr = self.term()?;
        while self.match_token(vec![Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expression::Binary {
                line: operator.line,
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> anyhow::Result<Expression> {
        let mut expr = self.factor()?;
        while self.match_token(vec![Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expression::Binary {
                line: operator.line,
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> anyhow::Result<Expression> {
        let mut expr = self.unary()?;
        while self.match_token(vec![Slash, Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expression::Binary {
                line: operator.line,
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> anyhow::Result<Expression> {
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

    fn primary(&mut self) -> anyhow::Result<Expression> {
        Ok(if self.match_token(vec![False]) {
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
                value: Value::I64(self.previous().lexeme.parse()?),
            }
        } else if self.match_token(vec![FloatingPoint]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: FloatingPoint,
                value: Value::F64(self.previous().lexeme.parse()?),
            }
        } else if self.match_token(vec![Text]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Text,
                value: Value::String(self.previous().lexeme.to_string()),
            }
        } else if self.match_token(vec![LeftParen]) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' after expression.")?;
            Expression::Grouping {
                line: self.peek().line,
                expression: Box::new(expr),
            }
        } else {
            let token = self.advance().clone();
            let (var_name, var_type) = self
                .vars
                .iter()
                .filter_map(|e| {
                    if let Variable { name, var_type, .. } = e {
                        Some((name, var_type))
                    } else {
                        None
                    }
                })
                .find(|e| e.0 == &token.lexeme)
                .ok_or_else(|| return anyhow::anyhow!("Unknown variable: {}", token.lexeme))?;
            Variable {
                name: var_name.to_string(),
                var_type: var_type.clone(),
                line: token.line,
            }
        })
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> anyhow::Result<Token> {
        if self.check(token_type) {
            self.advance();
        } else {
            self.had_error = true;
            return Err(anyhow::anyhow!(message.to_string()));
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
        self.peek().token_type == TokenType::Eof
    }
}

fn calculate_type(
    declared_type: Option<TokenType>,
    inferred_type: TokenType,
) -> anyhow::Result<TokenType> {
    Ok(if let Some(declared_type) = declared_type {
        if declared_type != inferred_type {
            match (declared_type, inferred_type) {
                (I32, I64) => I32,
                (U32, U64) => U32,
                (F32, F64) => F32,
                (F64, I64) => F64,
                (U64, I64) => U64,
                (U64, I32) => U64,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Incompatible types. Expected {}, found {}",
                        declared_type,
                        inferred_type
                    ));
                }
            }
        } else {
            declared_type
        }
    } else {
        match inferred_type{
            Integer => I64,
            FloatingPoint => F64,
            Text => Text,
            Bool => Bool,
            Date => Date,
            ListType => ListType,
            MapType => MapType,
            Object => Object,
            _ => panic!("Unexpected type"),
        }
    })
}

#[derive(Debug)]
pub enum Statement {
    ExpressionStmt {
        expression: Expression,
    },
    VarStmt {
        name: Token,
        var_type: TokenType,
        initializer: Expression,
    },
    Print {
        value: Expression,
    },
}

impl Statement {
    pub fn line(&self) -> usize {
        match self {
            Statement::ExpressionStmt { expression } => expression.line(),
            Statement::VarStmt {
                name,
                var_type,
                initializer,
            } => name.line,
            Statement::Print { value } => value.line(),
        }
    }
}

#[derive(Debug)]
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
    Variable {
        line: usize,
        name: String,
        var_type: TokenType,
    },
}

impl Expression {
    pub fn line(&self) -> usize {
        match self {
            Self::Binary { line, .. } => *line,
            Self::Unary { line, .. } => *line,
            Self::Grouping { line, expression } => *line,
            Self::Literal { line, .. } => *line,
            Self::Variable { line, .. } => *line,
        }
    }

    pub fn infer_type(&self) -> TokenType {
        match self {
            Self::Binary {
                line,
                left,
                operator,
                right,
            } => {
                let left_type = left.infer_type();
                let right_type = right.infer_type();
                if left_type == right_type {
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
                            (_, Text) => Text,
                            (Text, _) => Text,
                            (FloatingPoint, _) => F64,
                            (Integer, FloatingPoint) => F64,
                            (Integer, _) => I64,
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
            Self::Grouping { line, expression } => expression.infer_type(),
            Self::Literal { literaltype, .. } => literaltype.clone(),
            Self::Unary { right, .. } => right.infer_type(),
            Self::Variable { var_type, .. } => var_type.clone(),
        }
    }
}
