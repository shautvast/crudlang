use crate::{
    keywords,
    tokens::{
        Token,
        TokenType::{self},
    },
};
use crate::tokens::TokenType::BitXor;

pub fn scan(source: &str) -> Vec<Token> {
    let scanner = Scanner {
        chars: source.chars().collect(),
        current: 0,
        start: 0,
        line: 0,
        tokens: vec![],
        new_line: true,
    };
    scanner.scan()
}

impl Scanner {
    fn scan(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.add_token(TokenType::Eof);
        self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        if self.new_line && (c == ' ' || c == '\t') {
            self.add_token(TokenType::Indent);
            self.new_line = false;
        } else {
            if c != '\n' {
                self.new_line = false;
            }
            match c {
                '(' => self.add_token(TokenType::LeftParen),
                ')' => self.add_token(TokenType::RightParen),
                '{' => self.add_token(TokenType::LeftBrace),
                '}' => self.add_token(TokenType::RightBrace),
                '[' => self.add_token(TokenType::LeftBracket),
                ']' => self.add_token(TokenType::RightBracket),
                ',' => self.add_token(TokenType::Comma),
                '.' => self.add_token(TokenType::Dot),
                '-' => self.add_token(TokenType::Minus),
                '+' => self.add_token(TokenType::Plus),
                ':' => self.add_token(TokenType::Colon),
                '*' => self.add_token(TokenType::Star),
                '!' => {
                    let t = if self.match_next('=') {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    };
                    self.add_token(t);
                }
                '=' => {
                    let t = if self.match_next('=') {
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    };
                    self.add_token(t);
                }
                '<' => {
                    let t = if self.match_next('=') {
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    };
                    self.add_token(t)
                }
                '>' => {
                    let t = if self.match_next('=') {
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    };
                    self.add_token(t);
                }
                '/' => {
                    if self.match_next('/') {
                        // todo make distinction between comment and doc
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        self.add_token(TokenType::Slash);
                    }
                }
                '"' => self.string(),
                '\r' | '\t' | ' ' => {}
                '\n' => {
                    self.line += 1;
                    self.new_line = true;
                }
                '&' => {
                    let t = if self.match_next('&') {
                        TokenType::LogicalAnd
                    } else {
                        TokenType::BitAnd
                    };
                    self.add_token(t);
                }
                '|' => {
                    let t = if self.match_next('|') {
                        TokenType::LogicalOr
                    } else {
                        TokenType::BitOr
                    };
                    self.add_token(t);
                }
                '^' => self.add_token(BitXor),
                _ => {
                    if is_digit(c) {
                        self.number();
                    } else if is_alpha(c) {
                        self.identifier();
                    } else {
                        println!("Unexpected identifier at line {}", self.line);
                    }
                }
            }
        }
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }
        let value: String = self.chars[self.start..self.current].iter().collect();
        let tokentype = keywords::get_keyword(&value).unwrap_or(TokenType::Identifier);

        self.add_token_with_value(tokentype, value);
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();
        }

        while is_digit(self.peek()) {
            self.advance();
        }
        let value: String = self.chars[self.start..self.current].iter().collect();
        self.add_token_with_value(TokenType::Number, value);
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            println!("Unterminated string at {}", self.line)
        }

        self.advance();

        let value: String = self.chars[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token_with_value(TokenType::String, value);
    }

    fn peek(&self) -> char {
        if self.current>=self.chars.len(){
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    fn peek_next(&self) -> char {
        self.chars[self.current + 1]
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.chars[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn add_token(&mut self, tokentype: TokenType) {
        self.tokens
            .push(Token::new(tokentype, "".to_string(), self.line));
    }

    fn add_token_with_value(&mut self, tokentype: TokenType, value: String) {
        self.tokens.push(Token::new(tokentype, value, self.line));
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.chars[self.current - 1]
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}

struct Scanner {
    chars: Vec<char>,
    current: usize,
    start: usize,
    tokens: Vec<Token>,
    line: usize,
    new_line: bool,
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' || c == '$'
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn  test() {
        let tokens = scan(
            r#"struct Customer:
                        id: u32,
                        first_name: string,
                        last_name: string,
                        date_fetched: date,"#,
        );
        let tokenstring = format!("{:?}", tokens);
        println!("{}", tokenstring);
        // assert_eq!(tokenstring,r#"[Token { tokentype: Fn, lexeme: "fn", line: 2 }, Token { tokentype: Identifier, lexeme: "get", line: 2 }, Token { tokentype: LeftParen, lexeme: "", line: 2 }, Token { tokentype: Identifier, lexeme: "id", line: 2 }, Token { tokentype: Colon, lexeme: "", line: 2 }, Token { tokentype: Identifier, lexeme: "u32", line: 2 }, Token { tokentype: RightParen, lexeme: "", line: 2 }, Token { tokentype: Minus, lexeme: "", line: 2 }, Token { tokentype: Greater, lexeme: "", line: 2 }, Token { tokentype: Identifier, lexeme: "Customer", line: 2 }, Token { tokentype: Colon, lexeme: "", line: 2 }, Token { tokentype: Identifier, lexeme: "service", line: 3 }, Token { tokentype: Dot, lexeme: "", line: 3 }, Token { tokentype: Identifier, lexeme: "get", line: 3 }, Token { tokentype: LeftParen, lexeme: "", line: 3 }, Token { tokentype: Identifier, lexeme: "id", line: 3 }, Token { tokentype: RightParen, lexeme: "", line: 3 }]"#)
    }
}
