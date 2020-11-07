use itertools::{multipeek, MultiPeek};
use thiserror::Error;

use crate::parser::token::*;

pub type Result<T> = std::result::Result<T, ScanError>;
type CharIterator = Iterator<Item = char>;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("unterminated string")]
    UnterminatedString { start: SrcPos },
    #[error("unterminated commet")]
    UnterminatedComment { start: SrcPos },
    #[error("unexpected EoF")]
    UnexpectedEof(),
    #[error("unexpected character")]
    UnexpectedCharacter { found: char, at: SrcPos },
}

/// Wrapper of char iterator
struct ScanState<I: Iterator<Item = char>> {
    /// Iternally, it's using `MultiPeek` from itertools
    src: MultiPeek<I>,
    // Current source span
    sp: Span,
    // TODO: way to map span to SrcPo
}

impl<I> Iterator for ScanState<I>
where
    I: Iterator<Item = char>,
{
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let next = self.src.next();
        if let Some(c) = next {
            self.sp.hi.0 += 1;
            match c {
                '\n' => {
                    // self.pos.inc_ln();
                }
                _ => {}
            };
        }
        next
    }
}

impl<I: Iterator<Item = char>> ScanState<I> {
    pub fn new(iter: I) -> Self {
        Self {
            src: multipeek(iter),
            sp: Span::default(),
        }
    }

    pub fn safe_peek(&mut self) -> Option<&char> {
        self.src.reset_peek();
        self.src.peek()
    }

    pub fn peek_char(&mut self) -> Option<&char> {
        self.src.peek()
    }

    fn next_char(&mut self) -> Option<char> {
        let x = self.src.next();
        if x.is_some() {}
        x
    }

    fn consume_span(&mut self) -> Span {
        let sp = self.sp;
        self.sp.lo.0 = self.sp.hi.0;
        sp
    }
}

/// Parsers
impl<I: Iterator<Item = char>> ScanState<I> {
    // scanner functions
    pub fn ws(&mut self) -> Option<SpanToken> {
        while let Some(c) = self.src.peek() {
            match *c {
                ' ' | '\r' | '\t' => {
                    self.next();
                }
                '/' => {
                    match self.src.peek() {
                        Some(c) if *c == '/' => self.line_comment(),
                        Some(c) if *c == '*' => self.range_comment(),
                        _ => return Some(SpanToken::new(Token::Ws, self.consume_span())),
                    };
                }
                _ => break,
            };
        }

        None
    }

    pub fn line_comment(&mut self) {
        while let Some(c) = self.src.next() {
            if c == '\n' {
                break;
            }
        }
    }

    pub fn range_comment(&mut self) {
        self.src.next(); // consume `/` or `*` that was only peeked
        while let Some(c) = self.src.next() {
            match c {
                // TODO: this may cause error of multipeek
                '*' if self.src.peek() == Some(&'/') => {
                    return;
                }
                '/' if self.src.peek() == Some(&'*') => {
                    self.range_comment();
                }
                _ => {}
            }
        }
        // TODO: error for unterminated comments here
    }
}

/// A tokenizer of `Iterator<Item = char>`
pub struct Scanner<I: Iterator<Item = char>> {
    state: ScanState<I>,
}

impl<I: Iterator<Item = char>> Scanner<I> {
    pub fn new(iter: I) -> Self {
        Self {
            state: ScanState::new(iter),
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.state.ws();
        let c = match self.state.next() {
            None => return Ok(Token::Eof),
            Some(c) => c,
        };

        Ok(match c {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            '.' => Token::Dot,
            '-' => Token::Minus,
            '+' => Token::Plus,
            '/' => Token::Slash,
            '*' => Token::Star,
            '!' => self.one_two(Token::Bang, '=', Token::BangEqual),
            '=' => self.one_two(Token::Equal, '=', Token::EqualEqual),
            '<' => self.one_two(Token::Less, '=', Token::LessEqual),
            '>' => self.one_two(Token::Greater, '=', Token::GreaterEqual),
            _ => {
                unimplemented!()
                // return Err(ScanError::UnexpectedCharacter {
                //     found: c,
                //     at: self.state.pos(),
                // })
            }
        })
    }

    fn one_two(&mut self, not_match: Token, expected: char, if_match: Token) -> Token {
        match self.state.peek_char() {
            None => not_match,
            Some(n) if *n == expected => {
                self.state.next();
                if_match
            }
            _ => not_match,
        }
    }
}

// maybe you need:
// cargo test -- --test-threads 1 --nocapture
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        println!("=== test scanner ===");
        let src = "(){};,.-+/*<><=>=";
        let mut s = Scanner::new(src.chars());
        // let tks = s.iter()
    }
}
