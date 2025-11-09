use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum TokenType {
    Bang,
    BangEqual,
    BitAnd,
    Pipe,
    BitXor,
    Bool,
    Char,
    Colon,
    Comma,
    Date,
    Dot,
    Else,
    Eof,
    Eol,
    Equal,
    EqualEqual,
    Error,
    F32,
    F64,
    False,
    Fn,
    For,
    Greater,
    GreaterEqual,
    GreaterGreater,
    Hash,
    Hex,
    I32,
    I64,
    Identifier,
    If,
    Indent,
    Integer,
    SignedInteger,
    UnsignedInteger,
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
    FloatingPoint,
    Object,
    Plus,
    Print,
    Question,
    Return,
    RightParen,
    RightBrace,
    RightBracket,
    Semicolon,
    SingleRightArrow,
    Slash,
    Star,
    StringType,
    True,
    DateTime,
    U32,
    U64,
    Unknown,
    Void,
    While,
    ObjectType(String)
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::StringType => write!(f, "string"),
            TokenType::Date => write!(f, "date"),
            TokenType::Char => write!(f, "char"),
            TokenType::I32 => write!(f, "i32"),
            TokenType::I64 => write!(f, "i64"),
            TokenType::U32 => write!(f, "u32"),
            TokenType::U64 => write!(f, "u64"),
            TokenType::F32 => write!(f, "f32"),
            TokenType::F64 => write!(f, "f64"),
            TokenType::Bool => write!(f, "bool"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::BitAnd => write!(f, "&"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::BitXor => write!(f, "^"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Comma => write!(f, ","),
            TokenType::FloatingPoint => write!(f, "float"),
            TokenType::MapType => write!(f, "map"),
            TokenType::ListType => write!(f, "list"),
            TokenType::Dot => write!(f, "."),
            TokenType::Else => write!(f, "else"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Eol => write!(f, "EOL"),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Error => write!(f, "error"),
            TokenType::False => write!(f, "false"),
            TokenType::Fn => write!(f, "fn"),
            TokenType::For => write!(f, "for"),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::GreaterGreater => write!(f, ">>"),
            TokenType::Hash => write!(f, "#"),
            TokenType::Hex => write!(f, "0x"),
            TokenType::If => write!(f, "if"),
            TokenType::Identifier => write!(f, "identifier"),
            TokenType::Indent => write!(f, "indent"),
            TokenType::Integer => write!(f, "integer"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::LeftParen => write!(f, "("),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::LessLess => write!(f, "<<"),
            TokenType::Let => write!(f, "let"),
            TokenType::LogicalAnd => write!(f, "&&"),
            TokenType::LogicalOr => write!(f, "||"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Not => write!(f, "not"),
            TokenType::Object => write!(f, "object"),
            TokenType::ObjectType(_) => write!(f, "object"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Print => write!(f, "print"),
            TokenType::Question => write!(f, "?"),
            TokenType::Return => write!(f, "return"),
            TokenType::RightParen => write!(f, ")"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::SingleRightArrow => write!(f, "->"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
            TokenType::DateTime => write!(f, "t\""),
            TokenType::True => write!(f, "true"),
            TokenType::Unknown => write!(f, "?"),
            TokenType::Void => write!(f, "()"),
            TokenType::While => write!(f, "while"),
            TokenType::SignedInteger => write!(f, "i32/64"),
            TokenType::UnsignedInteger => write!(f, "u32/64"),

        }
    }
}

impl Eq for TokenType {}

impl TokenType {
    pub(crate) fn is_type(&self) -> bool {
        match self {
            TokenType::I32
            | TokenType::I64
            | TokenType::U32
            | TokenType::U64
            | TokenType::F32
            | TokenType::F64
            | TokenType::StringType
            | TokenType::Date
            | TokenType::Object
            | TokenType::ListType
            | TokenType::MapType
            | TokenType::Char => true,
            _ => false,
        }
    }
}
