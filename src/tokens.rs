#[derive(Debug)]
pub struct Token {
    pub tokentype: TokenType,
    pub lexeme: String,
    pub line: usize,
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

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub(crate) enum TokenType {
    Error,
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
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitOr,
    BitXor,
    Fn,
    Struct,
    Else,
    False,
    True,
    Null,
    If,
    While,
    For,
    Return,
    Print,
    Eof,
    U32Type,
    U64Type,
    I32Type,
    I64Type,
    DateType,
    StringType,
}

impl Eq for TokenType {

}