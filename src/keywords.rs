use crate::tokens::TokenType;

pub(crate) fn get_keyword(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "and" => Some(TokenType::LogicalAnd),
        "bool" => Some(TokenType::Bool),
        "char" => Some(TokenType::Char),
        "date" => Some(TokenType::Date),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "f32" => Some(TokenType::F32),
        "f64" => Some(TokenType::F64),
        "fn" => Some(TokenType::Fn),
        "for" => Some(TokenType::For),
        "if" => Some(TokenType::If),
        "i32" => Some(TokenType::I32),
        "i64" => Some(TokenType::I64),
        "let" => Some(TokenType::Let),
        "list" => Some(TokenType::ListType),
        "map" => Some(TokenType::MapType),
        "or" => Some(TokenType::LogicalOr),
        "object" => Some(TokenType::Object),
        "print" => Some(TokenType::Print),
        "struct" => Some(TokenType::Struct),
        "string" => Some(TokenType::String),
        "true" => Some(TokenType::True),
        "u32" => Some(TokenType::U32),
        "u64" => Some(TokenType::U64),
        "while" => Some(TokenType::While),

        _ => None,
    }
}
