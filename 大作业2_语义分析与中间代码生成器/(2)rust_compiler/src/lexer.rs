/// Lexer: scans source text and produces a token stream
use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            input: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // skip until end of line
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        // already consumed /*
        let mut depth = 1;
        while depth > 0 {
            match self.advance() {
                Some('/') => {
                    if self.peek() == Some('*') {
                        self.advance();
                        depth += 1;
                    }
                }
                Some('*') => {
                    if self.peek() == Some('/') {
                        self.advance();
                        depth -= 1;
                    }
                }
                None => {
                    return Err(format!("{}:{}: 未闭合的块注释", self.line, self.col));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn read_number(&mut self, first: char) -> Token {
        let start_line = self.line;
        let start_col = self.col - 1; // first char already consumed
        let mut num_str = String::new();
        num_str.push(first);
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let value: i64 = num_str.parse().unwrap_or(0);
        Token::new(TokenKind::IntLiteral(value), start_line, start_col)
    }

    fn read_ident_or_keyword(&mut self, first: char) -> Token {
        let start_line = self.line;
        let start_col = self.col - 1;
        let mut ident = String::new();
        ident.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match ident.as_str() {
            "i32" => TokenKind::I32,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "mut" => TokenKind::Mut,
            "fn" => TokenKind::Fn,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "loop" => TokenKind::Loop,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            _ => TokenKind::Ident(ident),
        };
        Token::new(kind, start_line, start_col)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            let line = self.line;
            let col = self.col;

            match self.peek() {
                None => {
                    tokens.push(Token::new(TokenKind::Eof, line, col));
                    break;
                }
                Some('#') => {
                    self.advance();
                    tokens.push(Token::new(TokenKind::Eof, line, col));
                    break;
                }
                Some(c) => {
                    self.advance();
                    match c {
                        // Comments or division
                        '/' => {
                            if self.peek() == Some('/') {
                                self.advance();
                                self.skip_line_comment();
                                continue;
                            } else if self.peek() == Some('*') {
                                self.advance();
                                self.skip_block_comment()?;
                                continue;
                            } else {
                                tokens.push(Token::new(TokenKind::Slash, line, col));
                            }
                        }

                        // Operators
                        '+' => tokens.push(Token::new(TokenKind::Plus, line, col)),
                        '*' => tokens.push(Token::new(TokenKind::Star, line, col)),
                        '&' => tokens.push(Token::new(TokenKind::Ampersand, line, col)),

                        '-' => {
                            if self.peek() == Some('>') {
                                self.advance();
                                tokens.push(Token::new(TokenKind::Arrow, line, col));
                            } else {
                                tokens.push(Token::new(TokenKind::Minus, line, col));
                            }
                        }

                        '=' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                tokens.push(Token::new(TokenKind::Eq, line, col));
                            } else {
                                tokens.push(Token::new(TokenKind::Assign, line, col));
                            }
                        }

                        '!' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                tokens.push(Token::new(TokenKind::Ne, line, col));
                            } else {
                                return Err(format!("{}:{}: 非法字符 '!'", line, col));
                            }
                        }

                        '<' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                tokens.push(Token::new(TokenKind::Le, line, col));
                            } else {
                                tokens.push(Token::new(TokenKind::Lt, line, col));
                            }
                        }

                        '>' => {
                            if self.peek() == Some('=') {
                                self.advance();
                                tokens.push(Token::new(TokenKind::Ge, line, col));
                            } else {
                                tokens.push(Token::new(TokenKind::Gt, line, col));
                            }
                        }

                        // Delimiters
                        '(' => tokens.push(Token::new(TokenKind::LParen, line, col)),
                        ')' => tokens.push(Token::new(TokenKind::RParen, line, col)),
                        '{' => tokens.push(Token::new(TokenKind::LBrace, line, col)),
                        '}' => tokens.push(Token::new(TokenKind::RBrace, line, col)),
                        '[' => tokens.push(Token::new(TokenKind::LBracket, line, col)),
                        ']' => tokens.push(Token::new(TokenKind::RBracket, line, col)),

                        // Separators
                        ';' => tokens.push(Token::new(TokenKind::Semicolon, line, col)),
                        ':' => tokens.push(Token::new(TokenKind::Colon, line, col)),
                        ',' => tokens.push(Token::new(TokenKind::Comma, line, col)),

                        // Dot
                        '.' => {
                            if self.peek() == Some('.') {
                                self.advance();
                                tokens.push(Token::new(TokenKind::DotDot, line, col));
                            } else {
                                tokens.push(Token::new(TokenKind::Dot, line, col));
                            }
                        }

                        // Numbers
                        '0'..='9' => {
                            let tok = self.read_number(c);
                            tokens.push(tok);
                        }

                        // Identifiers / keywords
                        'a'..='z' | 'A'..='Z' | '_' => {
                            let tok = self.read_ident_or_keyword(c);
                            tokens.push(tok);
                        }

                        _ => {
                            return Err(format!("{}:{}: 非法字符 '{}'", line, col, c));
                        }
                    }
                }
            }
        }
        Ok(tokens)
    }
}
