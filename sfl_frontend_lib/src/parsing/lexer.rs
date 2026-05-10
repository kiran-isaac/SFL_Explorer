use std::fmt::Debug;

pub use super::token::*;

pub struct LexerError {
    pub e: String,
    pub line: usize,
    pub col: usize,
}

impl Debug for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Lexer Error at [{}:{}]: {}",
            self.line + 1,
            self.col + 1,
            self.e
        )
    }
}

pub struct Lexer {
    file: Vec<char>,
    #[allow(dead_code)]
    filename: Option<String>,
    i: usize,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(file: String, filename: Option<String>) -> Self {
        // Add a cheeky couple of null bytes to the end of the file to make sure we don't go out of bounds
        let vec = (file + &"\0".repeat(10)).chars().collect();
        Lexer {
            file: vec,
            filename,
            i: 0,
            line: 0,
            col: 0,
        }
    }

    #[inline(always)]
    fn c(&self) -> char {
        if self.i >= self.file.len() {
            return '\0';
        }
        self.file[self.i]
    }

    fn advance(&mut self) {
        self.col += 1;
        self.i += 1;
    }

    fn error(&self, msg: String) -> LexerError {
        LexerError {
            e: msg,
            line: self.line,
            col: self.col,
        }
    }

    #[inline(always)]
    fn is_id_char(&self, c: char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '+' | '-' | '/' | '*' | '%' | '\'' => true,
            _ => false,
        }
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn pos_string(&self) -> String {
        format!(
            "{}{}:{}",
            match &self.filename {
                None => "".to_string(),
                Some(f) => f.clone() + " ",
            },
            self.line,
            self.col
        )
    }

    fn parse_id(&mut self) -> Result<Token, LexerError> {
        let mut str = self.c().to_string();

        self.advance();

        while self.is_id_char(self.c()) {
            str.push(self.c());
            self.advance();
        }

        // catch KWs
        let tt = match str.as_str() {
            "true" | "false" => TokenType::BoolLit,
            "match" => TokenType::Match,
            "forall" => TokenType::Forall,
            "type" => TokenType::KWType,
            "data" => TokenType::KWData,
            _ => TokenType::Id,
        };

        Ok(Token { tt, value: str })
    }

    /// Hijack the parse_id function to parse type ids and then
    /// change the TokenType to TypeId
    fn lex_type_id(&mut self) -> Result<Token, LexerError> {
        Ok(Token {
            tt: TokenType::UppercaseId,
            value: self.parse_id()?.value,
        })
    }

    fn lex_num_lit(&mut self) -> Result<Token, LexerError> {
        let mut str = String::new();

        let mut has_point = false;
        let mut char_before_point = false;
        let mut char_after_point = false;

        match self.c() {
            '-' => {
                str.push(self.c());
                self.advance();
            }
            _ => {}
        }

        while !(self.c().is_whitespace() || self.c() == '\0' || self.c() == ')' || self.c() == ',')
        {
            match self.c() {
                '0'..='9' => {
                    if has_point {
                        char_after_point = true;
                    } else {
                        char_before_point = true;
                    }
                    str.push(self.c());
                }
                '.' => {
                    if has_point {
                        return Err(self.error(format!("Unexpected char: {}", self.c())));
                    }
                    has_point = true;
                    str.push(self.c());
                }
                _ => {
                    return Err(self.error(format!("Unexpected char in num literal: {}", self.c())))
                }
            }

            self.advance();
        }

        if !(char_before_point || char_after_point) {
            return Err(self.error(format!("Empty number literal")));
        }

        if has_point {
            Ok(Token {
                tt: TokenType::FloatLit,
                value: str,
            })
        } else {
            Ok(Token {
                tt: TokenType::IntLit,
                value: str,
            })
        }
    }

    fn lex_char_lit(&mut self) -> Result<Token, LexerError> {
        let mut str = String::new();

        self.advance();

        while self.c() != '\'' {
            if self.c().is_ascii_control() {
                return Err(self.error(format!("Unexpected char in char literal: {}", self.c())));
            }
            str.push(self.c());
            self.advance();

            if str.len() > 2 {
                return Err(self.error(format!("Unterminated char literal")));
            }
        }

        self.advance();

        // convert str to char accounting for escape sequences
        let char = match str.as_str() {
            "\\n" => '\n',
            "\\t" => '\t',
            "\\r" => '\r',
            "\\0" => '\0',
            _ => {
                if str.len() == 2 {
                    // check if first char is a backslash
                    if str.chars().next().unwrap() == '\\' {
                        return Err(self.error(format!("Invalid escape sequence: {}", str)));
                    } else {
                        return Err(self.error(format!("Invalid char literal: {}", str)));
                    }
                }
                if str.len() == 0 {
                    return Err(self.error(format!("Empty char literal")));
                }

                str.chars().next().unwrap()
            }
        };

        Ok(Token {
            tt: TokenType::CharLit,
            value: char.to_string(),
        })
    }

    pub fn get_token(&mut self) -> Result<Token, LexerError> {
        // Advance, and if we hit a newline, return a newline token
        // If we hit multiple newlines, skip all but one
        // If we hit other whitespace, skip it
        while self.i < self.file.len() && self.c().is_whitespace() {
            if self.c() == '\n' {
                while self.c() == '\n' {
                    self.line += 1;
                    self.col = 0;
                    self.i += 1;
                }

                return Ok(Token {
                    tt: TokenType::Newline,
                    value: "\n".to_string(),
                });
            } else {
                self.advance();
            }
        }
        let c = self.c();

        match c {
            'a'..='z' | '_' | '+' | '*' | '%' => self.parse_id(),
            'A'..='Z' => self.lex_type_id(),
            '0'..='9' => self.lex_num_lit(),
            '-' => match self.file[self.i + 1] {
                '>' => {
                    self.advance();
                    self.advance();
                    Ok(Token {
                        tt: TokenType::RArrow,
                        value: "->".to_string(),
                    })
                }
                '0'..='9' | '.' => self.lex_num_lit(),
                _ => self.parse_id(),
            },
            '.' => match self.file[self.i + 1] {
                '0'..='9' => self.lex_num_lit(),
                _ => {
                    self.advance();
                    Ok(Token {
                        tt: TokenType::Dot,
                        value: ".".to_string(),
                    })
                }
            },
            '(' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::LParen,
                    value: "(".to_string(),
                })
            }
            '|' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::Bar,
                    value: "|".to_string(),
                })
            }
            '/' => {
                match self.file[self.i + 1] {
                    '/' => {
                        self.advance();
                        while self.c() != '\n' && self.c() != '\0' {
                            self.advance();
                        }
                    }
                    '*' => {
                        self.advance();
                        self.advance();
                        while !(self.c() == '*' && self.file[self.i + 1] == '/') {
                            if self.c() == '\n' {
                                self.line += 1;
                                self.col = 0;
                                self.i += 1;
                            } else if self.c() == '\0' {
                                return Err(self.error(format!("Unterminated block comment")));
                            } else {
                                self.advance();
                            }
                        }
                        self.advance();
                        self.advance();
                    }
                    _ => return self.parse_id(),
                }
                self.get_token()
            }
            ':' => {
                self.advance();
                match self.c() {
                    ':' => {
                        self.advance();
                        Ok(Token {
                            tt: TokenType::DoubleColon,
                            value: "::".to_string(),
                        })
                    }
                    _ => Err(self.error(format!("Unexpected char: {}", self.c()))),
                }
            }
            ',' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::Comma,
                    value: ",".to_string(),
                })
            }
            '\\' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::Lambda,
                    value: "\\".to_string(),
                })
            }
            '@' => {
                Err(self.error(format!("Unexpected char: {}", self.c())))
                // self.advance();
                // Ok(Token {
                //     tt: TokenType::Silence,
                //     value: "@".to_string(),
                // })
            }
            ')' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::RParen,
                    value: ")".to_string(),
                })
            }
            '{' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::LBrace,
                    value: "{".to_string(),
                })
            }
            '}' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::RBrace,
                    value: "}".to_string(),
                })
            }
            '$' => {
                self.advance();
                Ok(Token {
                    tt: TokenType::Dollar,
                    value: "$".to_string(),
                })
            }
            '=' => {
                self.advance();
                match self.c() {
                    '=' => {
                        self.advance();
                        Ok(Token {
                            tt: TokenType::Id,
                            value: "==".to_string(),
                        })
                    }
                    _ => Ok(Token {
                        tt: TokenType::Assignment,
                        value: "=".to_string(),
                    }),
                }
            }
            '<' => {
                self.advance();
                match self.c() {
                    '=' => {
                        self.advance();
                        Ok(Token {
                            tt: TokenType::Id,
                            value: "<=".to_string(),
                        })
                    }
                    _ => Ok(Token {
                        tt: TokenType::Id,
                        value: "<".to_string(),
                    }),
                }
            }
            '>' => {
                self.advance();
                match self.c() {
                    '=' => {
                        self.advance();
                        Ok(Token {
                            tt: TokenType::Id,
                            value: ">=".to_string(),
                        })
                    }
                    _ => Ok(Token {
                        tt: TokenType::Id,
                        value: ">".to_string(),
                    }),
                }
            }
            '\'' => self.lex_char_lit(),
            '\0' => Ok(Token {
                tt: TokenType::EOF,
                value: "".to_string(),
            }),
            _ => Err(self.error(format!("Unexpected char: {}", self.c()))),
        }
    }
}
