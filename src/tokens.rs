#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub(crate) fn new(tokentype: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type: tokentype,
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
    Bang,
    BangEqual,
    BitAnd,
    BitOr,
    BitXor,
    BoolType,
    CharType,
    Colon,
    Comma,
    DateType,
    Dot,
    Else,
    Eof,
    Eol,
    Equal,
    EqualEqual,
    Error,
    False,
    Fn,
    For,
    Greater,
    GreaterEqual,
    GreaterGreater,
    Hash,
    I32Type,
    I64Type,
    If,
    Indent,
    Identifier,
    LeftBrace,
    LeftBracket,
    LeftParen,
    Less,
    LessEqual,
    LessLess,
    Let,
    ListType,
    MapType,
    LogicalAnd,
    LogicalOr,
    Minus,
    Not,
    Number,
    ObjectType,
    Plus,
    Print,
    Return,
    RightParen,
    RightBrace,
    RightBracket,
    Semicolon,
    Slash,
    Star,
    String,
    StringType,
    Struct,
    True,
    U32Type,
    U64Type,
    While,
}

impl Eq for TokenType {

}