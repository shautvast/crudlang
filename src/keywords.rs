use crate::tokens::TokenType;

pub(crate) fn get_keyword(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "fn" => Some(TokenType::Fn),
        "struct" => Some(TokenType::Struct),
        _ => None,
    }
}
