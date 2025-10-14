#[derive(Debug)]
pub(crate) struct Token {
    tokentype: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    pub(crate) fn new(tokentype: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            tokentype,
            lexeme,
            line,
        }
    }
}

#[derive(Debug)]
enum Value {
    None,
}

#[derive(Debug)]
pub(crate) enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    Dot,
    Star,
    Slash,
    Plus,
    Minus,
    Hash,
    Bang,
    BangEqual,
    EqualEqual,
    Equal,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Indent,
    Identifier,
    String,
    Number,
    Fn,
    Struct,
}
