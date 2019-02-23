use crate::token::{SourcePosition, SourceToken, Token};
use std::error;
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum ScannerError {
    UnterminatedString(SourcePosition),
}

pub struct Scanner<'s> {
    source: Peekable<Chars<'s>>,
    lexeme: String,
    position: SourcePosition,
}

impl<'s> Scanner<'s> {
    pub fn new(src: &'s str) -> Self {
        Self {
            source: src.chars().peekable(),
            lexeme: String::new(),
            position: SourcePosition::initial(),
        }
    }

    pub fn scan(&mut self) -> (Vec<SourceToken>, Vec<ScannerError>) {
        let mut tokens = Vec::<SourceToken>::new();
        let mut errors = Vec::<ScannerError>::new();
        loop {
            // None is just discarded
            let position = self.position;
            if let Some(result) = self.scan_token() {
                match result {
                    Ok(Token::Eof) => {
                        break;
                    }
                    Ok(token) => {
                        tokens.push(self.add_context(token, position));
                    }
                    Err(error) => {
                        errors.push(error);
                    }
                }
            };
        }

        return (tokens, errors);
    }

    fn add_context(&mut self, token: Token, position: SourcePosition) -> SourceToken {
        SourceToken::new(token, position, self.lexeme.clone())
    }

    fn is_at_end(&mut self) -> bool {
        self.source.peek() == None
    }

    fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }

    fn advance(&mut self) -> Option<char> {
        let next = self.source.next();
        if let Some(c) = next {
            self.lexeme.push(c);
            if c == '\n' {
                self.position.inc_line();
                self.position.init_column();
            } else {
                self.position.inc_column();
            }
        }
        return next;
    }

    fn advance_if_match(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance_until(&mut self, c: char) {
        match self.peek() {
            Some(_c) if _c == &c => {
                return;
            }
            None => {
                return;
            }
            _ => {
                self.advance();
            }
        };
    }

    /// Entry point of recursive scanning.
    /// Returns None for tokens to be discarded.
    fn scan_token(&mut self) -> Option<Result<Token, ScannerError>> {
        use Token::*;
        self.lexeme.clear();
        let c = match self.advance() {
            None => return Some(Ok(Eof)),
            Some(x) => x,
        };
        let _result = match c {
            '(' => Ok(LeftParen),
            ')' => Ok(RightParen),
            '{' => Ok(LeftBrace),
            '}' => Ok(RightBrace),
            ',' => Ok(Comma),
            '.' => Ok(Dot),
            '+' => Ok(Plus),
            '-' => Ok(Minus),
            ';' => Ok(Semicolon),
            '*' => Ok(Star),
            '!' => Ok(self.scan_operator('=', BangEqual, Bang)),
            '=' => Ok(self.scan_operator('=', EqualEqual, Equal)),
            '<' => Ok(self.scan_operator('=', LessEqual, Less)),
            '>' => Ok(self.scan_operator('=', GreaterEqual, Greater)),
            '/' => {
                if self.advance_if_match('/') {
                    self.advance_until('\n');
                    return None;
                } else {
                    Ok(Slash)
                }
            }
            ' ' | '\r' | '\t' => return None,
            '"' => self.scan_string(),
            _ => return None,
        };

        return Some(_result);
    }

    fn scan_operator(&mut self, expected: char, if_true: Token, if_false: Token) -> Token {
        if self.peek() == Some(&expected) {
            self.advance();
            if_true
        } else {
            if_false
        }
    }

    fn scan_string(&mut self) -> Result<Token, ScannerError> {
        loop {
            match self.advance() {
                None => return Err(ScannerError::UnterminatedString(self.position)),
                Some('"') => {
                    // remove both " characters
                    let len = self.lexeme.len() - 2;
                    return Ok(Token::String(
                        // self.lexeme.chars().skip(1).take(len).collect(),
                        self.lexeme[1..len].to_string(),
                    ));
                }
                _ => {}
            };
        }
    }
}
