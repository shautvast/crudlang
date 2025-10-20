use crate::tokens::TokenType;

pub(crate) fn get_keyword(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "fn" => Some(TokenType::Fn),
        "struct" => Some(TokenType::Struct),
        "u32" => Some(TokenType::U32Type),
        "string" => Some(TokenType::StringType),
        "date" => Some(TokenType::DateType),
        "print" => Some(TokenType::Print),
        "and" => Some(TokenType::LogicalAnd),
        "else" => Some(TokenType::Else),
        "false" => Some(TokenType::False),
        "true" => Some(TokenType::True),
        "for" => Some(TokenType::For),
        "if" => Some(TokenType::If),
        "or" => Some(TokenType::LogicalOr),
        "while" => Some(TokenType::While),

        _ => None,
    }
}
