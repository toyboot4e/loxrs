use std::fmt;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct BytePos(pub usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Span {
    pub lo: BytePos,
    pub hi: BytePos,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            lo: BytePos(0),
            hi: BytePos(0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // loss-less
    Ws,
    LineComment,
    LRangeComment,
    RRangeComment,

    // meta symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,

    // arithmetic
    Minus,
    Plus,
    // MinusEqual,
    // PlusEqual,
    Semicolon,
    // slash / star vs mul / div
    Slash,
    Star,

    // comparison
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // primitives
    Identifier,
    // literals
    String,
    Number,

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

/// Human friendly source position representation
#[derive(Debug, Clone, Copy)]
pub struct SrcPos {
    /// One-based line number
    ln: usize,
    /// One-based column number
    col: usize,
}

impl SrcPos {
    pub fn initial() -> Self {
        Self::new(1, 1)
    }

    pub fn new(ln: usize, col: usize) -> Self {
        Self { ln, col }
    }

    pub fn ln(&self) -> usize {
        self.ln
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

impl SrcPos {
    pub fn inc_ln(&mut self) {
        self.ln += 1;
        self.col = 1;
    }

    pub fn inc_col(&mut self) {
        self.col += 1;
    }
}

impl fmt::Display for SrcPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(ln:{} col]{})", self.ln, self.col)
    }
}

/// [`Token`] with span
pub struct SpanToken {
    pub tk: Token,
    pub sp: Span,
}

impl SpanToken {
    pub fn new(tk: Token, sp: Span) -> Self {
        Self { tk, sp }
    }
}

/// [`Token`] at source code position
pub struct SrcToken {
    pub tk: Token,
    pub pos: SrcPos,
}

// use std::fmt::{Debug, Formatter, Result};
// impl Debug for SpanToken {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//         write!(
//             f,
//             r##"{:3}:{:3}  {:?} ["{}"]"##,
//             self.pos.ln, self.pos.col, self.token, self.lexeme
//         )
//     }
// }
