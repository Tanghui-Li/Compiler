/// Token types for the Rust-like language lexer
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    I32,
    Let,
    If,
    Else,
    While,
    Return,
    Mut,
    Fn,
    For,
    In,
    Loop,
    Break,
    Continue,

    // Literals
    IntLiteral(i64),

    // Identifier
    Ident(String),

    // Assignment
    Assign, // =

    // Arithmetic operators
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /

    // Comparison operators
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=

    // Reference operators
    Ampersand, // &

    // Delimiters
    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Separators
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,

    // Special symbols
    Arrow,    // ->
    Dot,      // .
    DotDot,   // ..

    // End of file
    Eof,      // #
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::IntLiteral(n) => write!(f, "{}", n),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Eq => write!(f, "=="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Ge => write!(f, ">="),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

impl TokenKind {
    pub fn category(&self) -> &str {
        match self {
            TokenKind::I32 | TokenKind::Let | TokenKind::If | TokenKind::Else
            | TokenKind::While | TokenKind::Return | TokenKind::Mut | TokenKind::Fn
            | TokenKind::For | TokenKind::In | TokenKind::Loop | TokenKind::Break
            | TokenKind::Continue => "关键字",

            TokenKind::IntLiteral(_) => "整数",
            TokenKind::Ident(_) => "标识符",
            TokenKind::Assign => "赋值号",

            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
            | TokenKind::Eq | TokenKind::Ne | TokenKind::Lt | TokenKind::Le
            | TokenKind::Gt | TokenKind::Ge | TokenKind::Ampersand => "算符",

            TokenKind::LParen | TokenKind::RParen | TokenKind::LBrace
            | TokenKind::RBrace | TokenKind::LBracket | TokenKind::RBracket => "界符",

            TokenKind::Semicolon | TokenKind::Colon | TokenKind::Comma => "分隔符",

            TokenKind::Arrow | TokenKind::Dot | TokenKind::DotDot => "特殊符号",

            TokenKind::Eof => "结束符",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {}:{})", self.kind.category(), self.kind, self.line, self.col)
    }
}
