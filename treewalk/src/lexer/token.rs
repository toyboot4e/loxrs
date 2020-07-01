// maybe I should use `ByteSpan` and it comes in bytecode interpreter

pub struct Token {
    // TODO: rename to kind
    pub kind: TokenKind,
    pub pos: Location,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, pos: Location, lexeme: String) -> Self {
        Self {
            kind: kind,
            pos: pos,
            // Required?
            lexeme: lexeme,
        }
    }
}

// TODO: use newtype (struct Identifier(String))
pub type Identifier = String;

/// Does NOT have `EoF` as a kind
#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
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
    BangEq,
    Eq,
    EqEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,

    Ident(Identifier),
    // yeah this is terrible
    Str(String),
    Num(f64),

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
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    ln: usize,
    col: usize,
}

impl Location {
    pub fn initial() -> Self {
        Self::new(1, 1)
    }

    pub fn new(line: usize, column: usize) -> Self {
        Self {
            ln: line,
            col: column,
        }
    }

    pub fn ln(&self) -> usize {
        self.ln
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn inc_ln(&mut self) {
        self.ln += 1;
    }

    pub fn inc_col(&mut self) {
        self.col += 1;
    }

    pub fn init_col(&mut self) {
        self.col = 1;
    }
}

use std::fmt::{Debug, Formatter, Result};
impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            r##"{:3}:{:3}  {:?} ["{}"]"##,
            self.pos.ln, self.pos.col, self.kind, self.lexeme
        )
    }
}
