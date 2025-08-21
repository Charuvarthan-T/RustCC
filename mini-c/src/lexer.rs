// import the enum from the roken.rs 
use crate::token::Token;

// pub makes the Lexer struct accessible from other modules
pub struct Lexer {
    // entire source code as a list of characters
    // track current index in the input
    input: Vec<char>,
    position: usize,
}



impl Lexer {

    // constructor for the Lexer
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    // returns the character if available else none -> moves based on the index
    fn next_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            Some(ch)
        } else {
            None
        }
    }


    fn peek_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    // skips over spaces
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    // first skip spaces, and then return the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let ch = match self.next_char() {
            Some(c) => c,
            None => return Token::EOF,
        };

        // match the character and return the corresponding token
        // the core logic here
        // when a " is seen, it scans till the next " is seen
        match ch {
            ';' => Token::Semicolon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '=' => Token::Assign,
            ',' => Token::Comma,

            '"' => {
                let mut string_val = String::new();
                while let Some(next) = self.peek_char() {
                    if next == '"' {
                        self.next_char(); // consume closing quote
                        break;
                    }
                    string_val.push(self.next_char().unwrap());
                }
                Token::String(string_val)
            }
            c if c.is_ascii_alphabetic() => {
                let mut ident = c.to_string();
                while let Some(next) = self.peek_char() {
                    if next.is_ascii_alphanumeric() {
                        ident.push(self.next_char().unwrap());
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "int" => Token::Int,
                    "return" => Token::Return,
                    _ => Token::Ident(ident),
                }
            }

            // numbers are taken now
            c if c.is_ascii_digit() => {
                let mut number = c.to_string();
                while let Some(next) = self.peek_char() {
                    if next.is_ascii_digit() {
                        number.push(self.next_char().unwrap());
                    } else {
                        break;
                    }
                }
                Token::Number(number.parse::<i64>().unwrap())
            }

            // end it
            _ => Token::EOF,
        }
    }
}
