use crate::ast_compiler::Expression::{
    FunctionCall, NamedParameter, Stop, Variable,
};
use crate::errors::CompilerError::{
    self, Expected, IncompatibleTypes, ParseError, TooManyParameters, TypeError,
    UndeclaredVariable, UnexpectedIndent, UninitializedVariable,
};
use crate::errors::CompilerErrorAtLine;
use crate::tokens::TokenType::{
    Bang, Bool, Char, Colon, Date, Dot, Eof, Eol, Equal, F32, F64, False, FloatingPoint, Fn,
    Greater, GreaterEqual, GreaterGreater, I32, I64, Identifier, Indent, Integer, LeftBrace,
    LeftBracket, LeftParen, Less, LessEqual, LessLess, Let, ListType, MapType, Minus, Object, Plus,
    Print, RightBrace, RightBracket, RightParen, SignedInteger, SingleRightArrow, Slash, Star,
    StringType, True, U32, U64, UnsignedInteger,
};
use crate::tokens::{Token, TokenType};
use crate::value::Value;
use log::debug;

pub fn compile(
    path: Option<&str>,
    tokens: Vec<Token>,
) -> Result<Vec<Statement>, CompilerErrorAtLine> {
    let mut compiler = AstCompiler::new(path.unwrap_or(""), tokens);
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
    indent: Vec<usize>,
}

impl AstCompiler {
    fn new(_name: &str, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            had_error: false,
            indent: vec![0],
        }
    }

    fn reset(&mut self) {
        self.current = 0;
    }

    fn compile_tokens(&mut self) -> Result<Vec<Statement>, CompilerErrorAtLine> {
        self.reset();
        self.compile()
    }

    fn compile(&mut self) -> Result<Vec<Statement>, CompilerErrorAtLine> {
        self.current_line();
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
            debug!("AST {:?}", statements);
            Ok(statements)
        } else {
            Err(self.raise(CompilerError::Failure))
        }
    }

    fn raise(&self, error: CompilerError) -> CompilerErrorAtLine {
        CompilerErrorAtLine::raise(error, self.current_line())
    }

    fn indent(&mut self) -> Result<Option<Statement>, CompilerErrorAtLine> {
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
            Err(self.raise(UnexpectedIndent(indent_on_line, expected_indent)))
        } else if indent_on_line < expected_indent {
            self.indent.pop();
            return Ok(None);
        } else {
            Ok(Some(self.declaration()?))
        }
    }

    fn declaration(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        if self.match_token(vec![Fn]) {
            self.function_declaration()
        } else if self.match_token(vec![Let]) {
            self.let_declaration()
        } else if self.match_token(vec![Object]) {
            self.object_declaration()
        } else if self.match_token(vec![TokenType::Pipe]) {
            self.guard_declaration()
        } else {
            self.statement()
        }
    }

    //  | /. -> service.get_all()
    //  | /{uuid} -> service.get(uuid)?
    //  | ?{query.firstname} -> service.get_by_firstname(fname)?
    fn guard_declaration(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        let if_expr = self.guard_if_expr()?;
        let then_expr = self.expression()?;
        Ok(Statement::GuardStatement { if_expr, then_expr })
    }

    fn guard_if_expr(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        while !self.check(SingleRightArrow) {
            if self.match_token(vec![Slash]) {
                return self.path_guard_expr();
            } else if self.match_token(vec![TokenType::Question]) {
                return self.query_guard_expr();
            } else {
                return Err(self.raise(Expected("-> or ?")));
            }
        }
        Ok(Stop {
            line: self.peek().line,
        })
    }

    fn query_guard_expr(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        if self.match_token(vec![LeftBrace]) {
            let query_params = self.expression()?;
            self.consume(RightBrace, Expected("'}' after guard expression."))?;
            Ok(query_params)
        } else {
            Ok(Stop {
                line: self.peek().line,
            })
        }
    }

    fn path_guard_expr(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        if self.match_token(vec![LeftBrace]) {
            let path_params = self.match_expression()?;
            self.consume(RightBrace, Expected("'}' after guard expression."))?;
            Ok(path_params)
        } else {
            Ok(Stop {
                line: self.peek().line,
            })
        }
    }

    fn match_expression(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        Err(self.raise(Expected("unimplemented")))
    }

    fn object_declaration(&mut self) -> Result<Statement, CompilerErrorAtLine> {
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
                let field_type = self.peek().token_type.clone();
                if field_type.is_type() {
                    self.advance();
                } else {
                    Err(self.raise(Expected("a type")))?
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

    fn function_declaration(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        let name_token = self.consume(Identifier, Expected("function name."))?;
        self.consume(LeftParen, Expected("'(' after function name."))?;
        let mut parameters = vec![];
        while !self.check(RightParen) {
            if parameters.len() >= 25 {
                return Err(self.raise(TooManyParameters));
            }
            let parm_name = self.consume(Identifier, Expected("a parameter name."))?;

            self.consume(Colon, Expected(": after parameter name"))?;
            let var_type = self.peek().token_type.clone();

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
            self.advance().token_type.clone()
        } else {
            TokenType::Void
        };
        self.consume(Colon, Expected("colon (:) after function declaration."))?;
        self.consume(Eol, Expected("end of line."))?;

        let current_indent = self.indent.last().unwrap();
        self.indent.push(current_indent + 1);

        let body = self.compile()?;

        let function = Function {
            name: name_token.clone(),
            parameters,
            return_type,
            body,
        };

        Ok(Statement::FunctionStmt { function })
    }

    fn let_declaration(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        if self.peek().token_type.is_type() {
            return Err(self.raise(CompilerError::KeywordNotAllowedAsIdentifier(
                self.peek().token_type.clone(),
            )));
        }
        let name_token = self.consume(Identifier, Expected("variable name."))?;

        let declared_type = if self.check(Colon) {
            self.advance();
            Some(self.advance().token_type.clone())
        } else {
            None
        };

        if self.match_token(vec![Equal]) {
            let initializer = self.expression()?;
            self.consume(Eol, Expected("end of line after initializer."))?;

            // let inferred_type = initializer.infer_type();
            // let var_type = match calculate_type(declared_type, inferred_type) {
            //     Ok(var_type) => var_type,
            //     Err(e) => {
            //         self.had_error = true;
            //         return Err(self.raise(TypeError(Box::new(e))));
            //     }
            // };
            // self.vars.push(Variable {
            //     name: name_token.lexeme.to_string(),
            //     var_type,
            //     line: name_token.line,
            // });
            Ok(Statement::VarStmt {
                name: name_token,
                var_type: declared_type.unwrap_or(TokenType::Unknown),
                initializer,
            })
        } else {
            Err(self.raise(UninitializedVariable))?
        }
    }

    fn statement(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        if self.match_token(vec![Print]) {
            self.print_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        let expr = self.expression()?;
        self.consume(Eol, Expected("end of line after print statement."))?;
        Ok(Statement::PrintStmt { value: expr })
    }

    fn expr_statement(&mut self) -> Result<Statement, CompilerErrorAtLine> {
        let expr = self.expression()?;
        if !self.is_at_end() {
            self.consume(Eol, Expected("end of line after expression."))?;
        }
        Ok(Statement::ExpressionStmt { expression: expr })
    }

    fn expression(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        self.or()
    }

    fn or(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.and()?;
        self.binary(vec![TokenType::LogicalOr], expr)
    }

    fn and(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.bit_and()?;
        self.binary(vec![TokenType::LogicalAnd], expr)
    }

    fn bit_and(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.bit_or()?;
        self.binary(vec![TokenType::BitAnd], expr)
    }

    fn bit_or(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.bit_xor()?;
        self.binary(vec![TokenType::Pipe], expr)
    }

    fn bit_xor(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.equality()?;
        self.binary(vec![TokenType::BitXor], expr)
    }

    fn equality(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.comparison()?;
        self.binary(vec![TokenType::EqualEqual, TokenType::BangEqual], expr)
    }

    fn comparison(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.bitshift()?;
        self.binary(vec![Greater, GreaterEqual, Less, LessEqual], expr)
    }

    fn bitshift(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.term()?;
        self.binary(vec![GreaterGreater, LessLess], expr)
    }

    fn term(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.factor()?;
        self.binary(vec![Minus, Plus], expr)
    }

    fn factor(&mut self) -> Result<Expression, CompilerErrorAtLine> {
        let expr = self.unary()?;
        self.binary(vec![Slash, Star], expr)
    }

    fn binary(
        &mut self,
        types: Vec<TokenType>,
        mut expr: Expression,
    ) -> Result<Expression, CompilerErrorAtLine> {
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

    fn unary(&mut self) -> Result<Expression, CompilerErrorAtLine> {
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

    fn primary(&mut self) -> Result<Expression, CompilerErrorAtLine> {
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
                value: Value::I64(
                    self.previous()
                        .lexeme
                        .parse()
                        .map_err(|e| self.raise(ParseError(format!("{:?}", e))))?,
                ),
            }
        } else if self.match_token(vec![U32]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Integer,
                value: Value::U32(
                    u32::from_str_radix(&self.previous().lexeme.trim_start_matches("0x"), 16)
                        .map_err(|e| self.raise(ParseError(format!("{:?}", e))))?,
                ),
            }
        } else if self.match_token(vec![U64]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: Integer,
                value: Value::U64(
                    u64::from_str_radix(&self.previous().lexeme.trim_start_matches("0x"), 16)
                        .map_err(|e| self.raise(ParseError(format!("{:?}", e))))?,
                ),
            }
        } else if self.match_token(vec![FloatingPoint]) {
            Expression::Literal {
                line: self.peek().line,
                literaltype: FloatingPoint,
                value: Value::F64(
                    self.previous()
                        .lexeme
                        .parse()
                        .map_err(|e| self.raise(ParseError(format!("{:?}", e))))?,
                ),
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
            // function call?
            if self.match_token(vec![LeftParen]) {
                self.function_call(token.lexeme)?
            } else if self.match_token(vec![Colon]) {
                self.named_parameter(&token)?
            } else if self.check(Dot) {
                // chain of variable or function lookups?
                let mut name = "/".to_string();
                name.push_str(&self.previous().lexeme);
                while self.match_token(vec![Dot]) {
                    name.push_str("/");
                    name.push_str(&self.peek().lexeme);
                    self.advance();
                }
                // chained function call?
                if self.match_token(vec![LeftParen]) {
                    self.function_call(name)?
                } else {
                    // empty line
                    return if self.match_token(vec![Eol, Eof]) {
                        Ok(Expression::Literal {
                            value: Value::Void,
                            literaltype: Object,
                            line: token.line,
                        })
                    } else {
                        Err(self.raise(UndeclaredVariable(token.lexeme.clone())))
                    };
                }
            } else {
                // none of the above, must be a variable lookup
                self.variable_lookup(&token)?
            }
        })
    }

    fn named_parameter(&mut self, name: &Token) -> Result<Expression, CompilerErrorAtLine> {
        let value = self.expression()?;
        let line = name.line;
        Ok(NamedParameter {
            name: name.clone(),
            value: Box::new(value),
            line,
        })
    }

    fn list(&mut self) -> Result<Expression, CompilerErrorAtLine> {
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

    fn map(&mut self) -> Result<Expression, CompilerErrorAtLine> {
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

    fn variable_lookup(&mut self, name: &Token) -> Result<Expression, CompilerErrorAtLine> {
        Ok(Variable {
            name: name.lexeme.to_string(),
            var_type: TokenType::Unknown,
            line: name.line,
        })
    }

    fn function_call(&mut self, name: String) -> Result<Expression, CompilerErrorAtLine> {
        let mut arguments = vec![];
        while !self.match_token(vec![RightParen]) {
            if arguments.len() >= 25 {
                return Err(self.raise(TooManyParameters));
            }
            let arg = self.expression()?;
            arguments.push(arg);
            if self.peek().token_type == TokenType::Comma {
                self.advance();
            } else {
                self.consume(RightParen, Expected("')' after arguments."))?;
                break;
            }
        }
        Ok(FunctionCall {
            line: self.peek().line,
            name,
            arguments,
        })
    }

    fn consume(
        &mut self,
        token_type: TokenType,
        message: CompilerError,
    ) -> Result<Token, CompilerErrorAtLine> {
        if self.check(token_type) {
            self.advance();
        } else {
            self.had_error = true;
            return Err(self.raise(message));
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

    fn current_line(&self) -> usize {
        self.peek().line
    }
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
    GuardStatement {
        if_expr: Expression,
        then_expr: Expression,
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
            Statement::GuardStatement { if_expr, .. } => if_expr.line(),
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
    },
    Stop {
        line: usize,
    },
    PathMatch {
        line: usize,
        condition: Box<Expression>,
    },
    NamedParameter {
        line: usize,
        name: Token,
        value: Box<Expression>,
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
            Variable { line, .. } => *line,
            FunctionCall { line, .. } => *line,
            Stop { line } => *line,
            Expression::PathMatch { line, .. } => *line,
            NamedParameter { line, .. } => *line,
        }
    }
}
