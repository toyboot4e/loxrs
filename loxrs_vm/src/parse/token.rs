use std::fmt;

use crate::parse::span::{BytePos, ByteSpan, SrcPos, SrcSpan};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // ----------------------------------------
    // wihtespace
    Ws,
    LineComment,

    RangeComment,

    // ----------------------------------------
    // symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    /// .
    Comma,
    /// .
    Dot,
    /// :
    Colon,
    /// ;
    Semicolon,

    // ----------------------------------------
    // arithmetic
    Minus,
    Plus,
    // MinusEqual,
    // PlusEqual,
    Slash,
    Star,

    // ----------------------------------------
    // comparison
    Eq,
    EqEq,
    Bang,
    BangEq,
    Gt,
    Ge,
    Lt,
    Le,

    // ----------------------------------------
    // primitives
    Ident,
    // literals
    Str,
    Num,

    // ----------------------------------------
    // keywords

    // control flow
    If,
    Else,
    For,
    While,
    Loop,

    // logical
    // And,
    // Or,

    // literals
    True,
    False,
    Nil,

    // keywords
    // Class,
    // Super,
    /// self
    SelfSmall,
    /// Self
    SelfCapital,

    // statements
    Return,
    Var,
    Fn,

    /// End of input
    Eof,
}

/// [`Token`] with span
#[derive(Clone, PartialEq)]
pub struct SpanToken {
    pub tk: Token,
    pub sp: ByteSpan,
}

impl SpanToken {
    pub fn new(tk: Token, sp: impl Into<ByteSpan>) -> Self {
        Self { tk, sp: sp.into() }
    }

    pub fn slice<'a>(&self, src: &'a str) -> &'a str {
        &src[self.sp.lo.0..self.sp.hi.0]
    }
}

impl fmt::Debug for SpanToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SpanToken({:?}, {})", self.tk, self.sp)?;
        Ok(())
    }
}

/// [`Token`] at source code position
#[derive(Clone, Debug, PartialEq)]
pub struct SrcToken {
    pub tk: Token,
    pub sp: SrcSpan,
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
