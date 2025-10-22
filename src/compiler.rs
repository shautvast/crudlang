use crate::chunk::Chunk;
use crate::scanner::scan;
use crate::tokens::{Token, TokenType};
use crate::value::Value;
use crate::vm::{
    OP_ADD, OP_BITAND, OP_BITOR, OP_BITXOR, OP_CONSTANT, OP_DEF_BOOL, OP_DEF_CHAR, OP_DEF_DATE,
    OP_DEF_F64, OP_DEF_I32, OP_DEF_I64, OP_DEF_LIST, OP_DEF_MAP, OP_DEF_STRUCT, OP_DEF_STRING,
    OP_DEFINE, OP_DIVIDE, OP_EQUAL, OP_FALSE, OP_GET, OP_GREATER, OP_GREATER_EQUAL, OP_LESS,
    OP_LESS_EQUAL, OP_MULTIPLY, OP_NEGATE, OP_NOT, OP_POP, OP_PRINT, OP_RETURN, OP_SHL, OP_SHR,
    OP_SUBTRACT, OP_TRUE,
};
use anyhow::anyhow;
use std::collections::HashMap;
use std::mem::discriminant;
use std::sync::LazyLock;
use tracing::debug;

pub fn compile(source: &str) -> anyhow::Result<Chunk> {
    let tokens = scan(source);
    debug!("Scanned tokens: {:?}", tokens);

    let mut compiler = Compiler {
        source: source.lines().map(|s| s.to_string()).collect(),
        chunk: Chunk::new("main"),
        previous_token: &tokens[0],
        current_token: &tokens[0],
        tokens: &tokens,
        current: 0,
        types: vec![],
        locals: vec![],
        previous: 0,
        had_error: false,
    };
    compiler.compile()
}

struct Compiler<'a> {
    source: Vec<String>,
    chunk: Chunk,
    tokens: &'a Vec<Token>,
    current: usize,
    previous_token: &'a Token,
    current_token: &'a Token,
    types: Vec<Token>,
    locals: Vec<String>,
    previous: usize,
    had_error: bool,
}

impl<'a> Compiler<'a> {
    fn compile(mut self) -> anyhow::Result<Chunk> {
        while !self.match_token(TokenType::Eof) {
            self.declaration()?;
        }

        // self.expression()?;

        // self.consume(TokenType::Eof, "Expect end of expression.")?;
        self.emit_byte(OP_RETURN);
        Ok(self.chunk)
    }

    fn declaration(&mut self) -> anyhow::Result<()> {
        if self.match_token(TokenType::Let) {
            self.let_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> anyhow::Result<()> {
        let index = self.parse_variable("Expect variable name")?;
        let mut declared_type = None;
        if self.check(TokenType::Colon) {
            self.consume(TokenType::Colon, "must not happen")?;
            match self.current_token.token_type {
                TokenType::I32
                | TokenType::I64
                | TokenType::U32
                | TokenType::U64
                | TokenType::Date
                | TokenType::String
                | TokenType::Char
                | TokenType::Bool
                | TokenType::ListType
                | TokenType::MapType => declared_type = Some(self.current_token.token_type),
                _ => return Err(anyhow!("Invalid type {:?}", self.current_token.token_type)),
            }
            self.advance()?;
        }
        if self.match_token(TokenType::Equal) {
            self.expression(declared_type)?;
            let derived_type = Some(&self.previous_token.token_type);
            self.consume(TokenType::Eol, "Expect end of line")?;
            self.define_variable(declared_type, derived_type, index)?;
        } else {
            return Err(anyhow!(
                "You cannot declare a variable without initializing it."
            ));
        }
        Ok(())
    }

    fn parse_variable(&mut self, message: &str) -> anyhow::Result<usize> {
        self.consume(TokenType::Identifier, message)?;
        self.identifier_constant(self.previous_token)
    }

    fn identifier_constant(&mut self, token: &Token) -> anyhow::Result<usize> {
        let name = token.lexeme.clone();
        let index = self.chunk.add_constant(Value::String(name));
        Ok(index)
    }

    fn define_variable(
        &mut self,
        var_type: Option<TokenType>,
        derived_type: Option<&TokenType>,
        index: usize,
    ) -> anyhow::Result<()> {
        let def_op = match var_type {
            Some(TokenType::I32) => OP_DEF_I32,
            Some(TokenType::I64) => OP_DEF_I64,
            Some(TokenType::U32) => OP_DEF_I64,
            Some(TokenType::U64) => OP_DEF_I64,
            Some(TokenType::Date) => OP_DEF_DATE,
            Some(TokenType::String) => OP_DEF_STRING,
            Some(TokenType::Char) => OP_DEF_CHAR,
            Some(TokenType::Bool) => OP_DEF_BOOL,
            Some(TokenType::ListType) => OP_DEF_LIST,
            Some(TokenType::MapType) => OP_DEF_MAP,
            Some(TokenType::Object) => OP_DEF_STRUCT,
            _ => match derived_type {
                Some(TokenType::Text) => OP_DEF_STRING,
                Some(TokenType::Bool) => OP_DEF_BOOL,
                Some(TokenType::Char) => OP_DEF_CHAR,
                Some(TokenType::F64) => OP_DEF_F64,
                Some(TokenType::I64) => OP_DEF_I64,
                Some(TokenType::ListType) => OP_DEF_LIST,
                Some(TokenType::MapType) => OP_DEF_MAP,
                _ => OP_DEFINE,
            },
        };

        self.emit_bytes(def_op, index as u16);
        Ok(())
    }

    fn statement(&mut self) -> anyhow::Result<()> {
        if self.match_token(TokenType::Print) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn expression_statement(&mut self) -> anyhow::Result<()> {
        debug!("expression statement");
        self.expression(None)?;
        self.emit_byte(OP_POP);
        Ok(())
    }

    fn print_statement(&mut self) -> anyhow::Result<()> {
        self.expression(None)?;
        self.consume(
            TokenType::Eol,
            "No further statements expected. Please start on a new line after the first one.\n",
        )?;
        self.emit_byte(OP_PRINT);
        Ok(())
    }

    fn advance(&mut self) -> anyhow::Result<()> {
        if self.current < self.tokens.len() - 1 {
            self.previous = self.current;
            self.previous_token = &self.tokens[self.previous];
            self.current += 1;
            self.current_token = &self.tokens[self.current];
        }
        if let TokenType::Error = self.current_token.token_type {
            self.had_error = true;
            Err(anyhow!(
                "Error at {} on line {}",
                self.current_token.lexeme,
                self.current_token.line
            ))
        } else {
            Ok(())
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> anyhow::Result<()> {
        if token_type == self.current_token.token_type {
            self.advance()
        } else {
            Err(anyhow!(
                r#"{} at line {}: "{}""#,
                message,
                self.current_token.line + 1,
                self.source[self.current_token.line]
            ))
        }
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            false
        } else {
            self.advance().expect("token expected");
            true
        }
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        self.current_token.token_type == token_type
    }

    fn expression(&mut self, expected_type: Option<TokenType>) -> anyhow::Result<()> {
        self.parse_precedence(PREC_ASSIGNMENT, expected_type)?;

        Ok(())
    }

    fn parse_precedence(
        &mut self,
        precedence: usize,
        expected_type: Option<TokenType>,
    ) -> anyhow::Result<()> {
        self.advance()?;
        let rule = get_rule(&self.previous_token.token_type);
        debug!("Precedence rule: {:?}", rule);
        if let Some(prefix) = rule.prefix {
            prefix(self, expected_type)?;
            while precedence <= get_rule(&self.current_token.token_type).precedence {
                self.advance()?;
                let infix_rule = get_rule(&self.previous_token.token_type).infix;
                if let Some(infix) = infix_rule {
                    infix(self, expected_type)?;
                }
            }
        } else {
            return Err(anyhow!("Expect expression."));
        }
        Ok(())
    }

    fn emit_byte(&mut self, byte: u16) {
        self.chunk.add(byte, self.previous_token.line);
    }

    fn emit_bytes(&mut self, b1: u16, b2: u16) {
        self.emit_byte(b1);
        self.emit_byte(b2);
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.chunk.add_constant(value);
        self.emit_bytes(OP_CONSTANT, index as u16);
    }
}

type ParseFn = fn(&mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()>;

#[derive(Debug)]
struct Rule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: usize,
}

impl Rule {
    fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: usize) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

fn number(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    let number = &s.previous_token.lexeme;
    let value = if let Some(expected_type) = expected_type {
        match expected_type {
            TokenType::I32 => Value::I32(number.parse()?),
            TokenType::I64 => Value::I64(number.parse()?),
            TokenType::U32 => Value::U32(number.parse()?),
            TokenType::U64 => Value::U64(number.parse()?),
            TokenType::F32 => Value::U32(number.parse()?),
            TokenType::F64 => Value::U64(number.parse()?),

            _ => {
                return Err(anyhow!(
                    "Invalid type: expected {} value, got {}({})",
                    expected_type,
                    &s.previous_token.token_type,
                    number
                ));
            }
        }
    } else {
        if let TokenType::Number = s.previous_token.token_type {
            if number.contains('.') {
                Value::F64(number.parse()?)
            } else {
                Value::I64(number.parse()?)
            }
        } else {
            return Err(anyhow!("I did not think this would happen"));
        }
    };
    s.emit_constant(value);
    Ok(())
}

fn literal(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    let actual_type = &s.previous_token.token_type;
    if let Some(expected_type) = expected_type {
        match (actual_type, expected_type) {
            (TokenType::False, TokenType::Bool) => s.emit_constant(Value::Bool(false)),
            (TokenType::True, TokenType::Bool) => s.emit_constant(Value::Bool(true)),
            (TokenType::Text, TokenType::String) => {
                s.emit_constant(Value::String(s.previous_token.lexeme.clone()))
            }
            _ => {
                return Err(anyhow!(
                    "Invalid type: expected {} value, got {}({})",
                    expected_type,
                    &s.previous_token.token_type,
                    s.previous_token.lexeme
                ));
            }
        }
    } else {
        match actual_type {
            TokenType::False => s.emit_constant(Value::Bool(false)),
            TokenType::True => s.emit_constant(Value::Bool(true)),
            TokenType::Text => s.emit_constant(Value::String(s.previous_token.lexeme.clone())),
            _ => {}
        }
    }
    Ok(())
}

fn skip(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    Ok(())
}

fn grouping(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    s.expression(None)?;
    s.consume(TokenType::RightParen, "Expect ')' after expression.")
}

fn unary(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    let operator_type = s.previous_token.token_type;

    s.parse_precedence(PREC_UNARY, None)?;

    match operator_type {
        TokenType::Minus => {
            s.emit_byte(OP_NEGATE);
        }
        TokenType::Bang => {
            s.emit_byte(OP_NOT);
        }
        _ => unimplemented!("unary other than ! and -"),
    }
    Ok(())
}

fn binary(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    let operator_type = &s.previous_token.token_type;
    debug!("operator {:?}", operator_type);
    let rule = get_rule(operator_type);
    s.parse_precedence(rule.precedence + 1, None)?;
    match operator_type {
        TokenType::Plus => s.emit_byte(OP_ADD),
        TokenType::Minus => s.emit_byte(OP_SUBTRACT),
        TokenType::Star => s.emit_byte(OP_MULTIPLY),
        TokenType::Slash => s.emit_byte(OP_DIVIDE),
        TokenType::BitAnd => s.emit_byte(OP_BITAND),
        TokenType::BitOr => s.emit_byte(OP_BITOR),
        TokenType::BitXor => s.emit_byte(OP_BITXOR),
        TokenType::GreaterGreater => s.emit_byte(OP_SHR),
        TokenType::LessLess => s.emit_byte(OP_SHL),
        TokenType::EqualEqual => s.emit_byte(OP_EQUAL),
        TokenType::Greater => s.emit_byte(OP_GREATER),
        TokenType::GreaterEqual => s.emit_byte(OP_GREATER_EQUAL),
        TokenType::Less => s.emit_byte(OP_LESS),
        TokenType::LessEqual => s.emit_byte(OP_LESS_EQUAL),
        _ => unimplemented!("binary other than plus, minus, star, slash"),
    }
    Ok(())
}

fn variable(s: &mut Compiler, expected_type: Option<TokenType>) -> anyhow::Result<()> {
    let index = s.identifier_constant(s.previous_token)?;
    s.emit_bytes(OP_GET, index as u16);
    Ok(())
}

fn get_rule(operator_type: &TokenType) -> &'static Rule {
    debug!("{:?}", operator_type);
    RULES.get(operator_type).unwrap()
}

static RULES: LazyLock<HashMap<TokenType, Rule>> = LazyLock::new(|| {
    let mut rules: HashMap<TokenType, Rule> = HashMap::new();
    rules.insert(TokenType::Bang, Rule::new(Some(unary), None, PREC_UNARY));
    rules.insert(TokenType::BangEqual, Rule::new(None, None, PREC_EQUALITY));
    rules.insert(TokenType::BitOr, Rule::new(None, Some(binary), PREC_BITOR));
    rules.insert(
        TokenType::BitXor,
        Rule::new(None, Some(binary), PREC_BITXOR),
    );
    rules.insert(TokenType::Colon, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Comma, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Date, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Dot, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Else, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Eof, Rule::new(Some(skip), None, PREC_NONE));
    rules.insert(TokenType::Eol, Rule::new(Some(skip), None, PREC_NONE));
    rules.insert(TokenType::Equal, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::False, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Fn, Rule::new(None, None, PREC_NONE));
    rules.insert(
        TokenType::EqualEqual,
        Rule::new(None, Some(binary), PREC_EQUALITY),
    );
    rules.insert(TokenType::False, Rule::new(Some(literal), None, PREC_NONE));
    rules.insert(
        TokenType::Greater,
        Rule::new(None, Some(binary), PREC_COMPARISON),
    );
    rules.insert(
        TokenType::GreaterEqual,
        Rule::new(None, Some(binary), PREC_COMPARISON),
    );
    rules.insert(
        TokenType::GreaterGreater,
        Rule::new(None, Some(binary), PREC_BITSHIFT),
    );
    rules.insert(TokenType::I32, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::I64, Rule::new(None, None, PREC_NONE));
    rules.insert(
        TokenType::Identifier,
        Rule::new(Some(variable), None, PREC_NONE),
    );
    rules.insert(TokenType::Indent, Rule::new(Some(skip), None, PREC_NONE));
    rules.insert(TokenType::LeftBrace, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::LeftBracket, Rule::new(None, None, PREC_NONE));
    rules.insert(
        TokenType::LeftParen,
        Rule::new(Some(binary), None, PREC_NONE),
    );
    rules.insert(
        TokenType::Less,
        Rule::new(None, Some(binary), PREC_COMPARISON),
    );
    rules.insert(
        TokenType::LessEqual,
        Rule::new(None, Some(binary), PREC_COMPARISON),
    );
    rules.insert(
        TokenType::LessLess,
        Rule::new(None, Some(binary), PREC_BITSHIFT),
    );
    rules.insert(
        TokenType::LogicalAnd,
        Rule::new(None, Some(binary), PREC_AND),
    );
    rules.insert(TokenType::LogicalOr, Rule::new(None, Some(binary), PREC_OR));
    rules.insert(
        TokenType::Minus,
        Rule::new(Some(unary), Some(binary), PREC_TERM),
    );
    rules.insert(TokenType::Number, Rule::new(Some(number), None, PREC_NONE));
    rules.insert(TokenType::Plus, Rule::new(None, Some(binary), PREC_TERM));
    rules.insert(TokenType::Print, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Return, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::RightParen, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::RightBrace, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::RightBracket, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Slash, Rule::new(None, Some(binary), PREC_FACTOR));
    rules.insert(TokenType::Star, Rule::new(None, Some(binary), PREC_FACTOR));
    rules.insert(TokenType::Text, Rule::new(Some(literal), None, PREC_NONE));
    rules.insert(
        TokenType::BitAnd,
        Rule::new(None, Some(binary), PREC_BITAND),
    );
    rules.insert(TokenType::String, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::Struct, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::True, Rule::new(Some(literal), None, PREC_NONE));
    rules.insert(TokenType::U32, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::U64, Rule::new(None, None, PREC_NONE));
    rules.insert(TokenType::While, Rule::new(None, None, PREC_NONE));

    rules
});

const PREC_NONE: usize = 0;
const PREC_ASSIGNMENT: usize = 1;
const PREC_OR: usize = 2;
const PREC_AND: usize = 3;
const PREC_BITAND: usize = 4;
const PREC_BITOR: usize = 5;
const PREC_BITXOR: usize = 6;
const PREC_EQUALITY: usize = 7;
const PREC_COMPARISON: usize = 8;
const PREC_BITSHIFT: usize = 9;
const PREC_TERM: usize = 10;
const PREC_FACTOR: usize = 11;
const PREC_UNARY: usize = 12;
const PREC_CALL: usize = 13;
const PREC_PRIMARY: usize = 14;

enum ValueType {
    DateType,
    BoolType,
    CharType,
    F32Type,
    F64Type,
    I32Type,
    I64Type,
    ObjectType,
    U32Type,
    U64Type,
    StringType,
    ListType,
    MapType,
}
