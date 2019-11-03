pub type Identifier = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    // MinusEqual,
    // PlusEqual,
    Semicolon,
    // slash / star vs mul / div
    Slash,
    Star,

    // one or more character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(Identifier),
    // literals
    String(String),
    Number(f64),

    // keywords
    And,
    Class,
    Self_,
    Else,
    False,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePosition {
    line: usize,
    column: usize,
}

impl SourcePosition {
    pub fn initial() -> Self {
        Self::new(1, 1)
    }

    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line: line,
            column: column,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn inc_line(&mut self) {
        self.line += 1;
    }

    pub fn inc_column(&mut self) {
        self.column += 1;
    }

    pub fn init_column(&mut self) {
        self.column = 1;
    }
}

/// [`Token`] in source code. Often referred to as `s_token`
pub struct SourceToken {
    // TODO: rename to kind
    pub token: Token,
    pub pos: SourcePosition,
    pub lexeme: String,
}

impl SourceToken {
    pub fn new(token: Token, pos: SourcePosition, lexeme: String) -> Self {
        Self {
            token: token,
            pos: pos,
            // Required?
            lexeme: lexeme,
        }
    }
}

use ::std::fmt::{Debug, Formatter, Result};
impl Debug for SourceToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            r##"{:3}:{:3}  {:?} ["{}"]"##,
            self.pos.line, self.pos.column, self.token, self.lexeme
        )
    }
}
