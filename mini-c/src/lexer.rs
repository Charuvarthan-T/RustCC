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
            // skip normal whitespace
            if ch.is_whitespace() {
                self.position += 1;
                continue;
            }

            // skip single-line comments starting with //
            if ch == '/' {
                // lookahead
                if let Some(next) = self.input.get(self.position + 1) {
                    if *next == '/' {
                        // consume '//' and then all chars until newline
                        self.position += 2;
                        while let Some(nc) = self.peek_char() {
                            self.position += 1;
                            if nc == '\n' {
                                break;
                            }
                        }
                        continue;
                    } else if *next == '*' {
                        // block comment /* ... */
                        self.position += 2; // consume '/*'
                        while let Some(_) = self.peek_char() {
                            // look for closing */
                            if let Some(c1) = self.peek_char() {
                                if c1 == '*' {
                                    // check next
                                    if let Some(c2) = self.input.get(self.position + 1) {
                                        if *c2 == '/' {
                                            // consume '*/'
                                            self.position += 2;
                                            break;
                                        }
                                    }
                                }
                                self.position += 1;
                            } else {
                                break;
                            }
                        }
                        continue;
                    }
                }
            }

            break;
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
                    "float" => Token::Float,
                    "char" => Token::Char,
                    "void" => Token::Void,
                    "return" => Token::Return,
                    _ => Token::Ident(ident),
                }
            }

            // numbers are taken now (integers and floats)
            c if c.is_ascii_digit() => {
                let mut number = c.to_string();
                let mut is_float = false;
                while let Some(next) = self.peek_char() {
                    if next.is_ascii_digit() {
                        number.push(self.next_char().unwrap());
                    } else if next == '.' && !is_float {
                        // float literal
                        is_float = true;
                        number.push(self.next_char().unwrap());
                        // collect fractional part
                        while let Some(frac) = self.peek_char() {
                            if frac.is_ascii_digit() {
                                number.push(self.next_char().unwrap());
                            } else {
                                break;
                            }
                        }
                        break;
                    } else {
                        break;
                    }
                }
                if is_float {
                    Token::FloatNumber(number.parse::<f64>().unwrap())
                } else {
                    Token::Number(number.parse::<i64>().unwrap())
                }
            }

            // char literal like 'a'
            '\'' => {
                // read char content; support simple escapes like '\n' or '\''
                let ch = if let Some(next) = self.next_char() {
                    if next == '\\' {
                        // escaped char
                        if let Some(escaped) = self.next_char() {
                            escaped
                        } else {
                            '\0'
                        }
                    } else {
                        next
                    }
                } else {
                    '\0'
                };
                // consume closing quote if present
                if let Some(peek) = self.peek_char() {
                    if peek == '\'' {
                        self.next_char();
                    }
                }
                Token::CharLiteral(ch)
            }

            // end it
            _ => Token::EOF,
        }
    }
}
