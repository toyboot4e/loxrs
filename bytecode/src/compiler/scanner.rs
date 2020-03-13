use ::itertools::{multipeek, MultiPeek};

use crate::compiler::token::{SourcePosition, Token};
use std::str::Chars;

type Result<T> = ::std::result::Result<T, ScanError>;

#[derive(Debug, Clone)]
pub enum ScanError {
    UnterminatedString(SourcePosition),
    UnterminatedComment(SourcePosition),
    UnexpectedEof(SourcePosition),
    UnexpectedCharacter(char, SourcePosition),
}

pub struct ScanState<I>
where
    I: Iterator<Item = char>,
{
    src: MultiPeek<I>,
    pos: SourcePosition,
    lexeme: String,
}

impl<I> ScanState<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(iter: I) -> Self {
        Self {
            src: multipeek(iter),
            pos: SourcePosition::initial(),
            lexeme: String::new(),
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        let c = match self.src.next() {
            None => return Ok(Token::Eof),
            Some(c) => c,
        };

        Ok(match c {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
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
            _ => return Err(ScanError::UnexpectedCharacter(c, self.pos)),
        })
    }

    fn one_two(&mut self, not_match: Token, expected: char, if_match: Token) -> Token {
        match self.src.peek() {
            None => not_match,
            Some(n) if *n == expected => {
                self.src.next();
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
    use crate::compiler::token::*;

    #[test]
    fn test_scanner() {
        println!("=== test scanner ===");
        let src = "(){};,.-+/*<><=>=";
        let mut s = ScanState::new(src.chars());
        loop {
            match s.next_token() {
                Ok(Token::Eof) => return,
                Ok(t) => println!("{:?}", t),
                Err(why) => {
                    eprintln!("{:?}", why);
                    return;
                }
            };
        }
    }
}
