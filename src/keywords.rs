use crate::tokens::TokenType;

pub(crate) fn get_keyword(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "and" => Some(TokenType::LogicalAnd),
        "bool" => Some(TokenType::BoolType),
        "char" => Some(TokenType::CharType),
        "date" => Some(TokenType::DateType),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "fn" => Some(TokenType::Fn),
        "for" => Some(TokenType::For),
        "if" => Some(TokenType::If),
        "i32" => Some(TokenType::I32Type),
        "i64" => Some(TokenType::I64Type),
        "let" => Some(TokenType::Let),
        "list" => Some(TokenType::ListType),
        "map" => Some(TokenType::MapType),
        "or" => Some(TokenType::LogicalOr),
        "object" => Some(TokenType::ObjectType),
        "print" => Some(TokenType::Print),
        "struct" => Some(TokenType::Struct),
        "string" => Some(TokenType::StringType),
        "true" => Some(TokenType::True),
        "u32" => Some(TokenType::U32Type),
        "u64" => Some(TokenType::U64Type),
        "while" => Some(TokenType::While),

        _ => None,
    }
}
