use itertools::{multipeek, MultiPeek};

use crate::abs::token::{SourcePosition, SourceToken, Token};
use std::str::Chars;

mod char_ext {
    pub fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    /// Returns true if it may be a beginning of an identifier
    /// i.e. an alphabet or an under score.
    pub fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    pub fn is_alphanumeric(c: char) -> bool {
        is_digit(c) || is_alpha(c)
    }

    /// Returns true if it's a char to be discarded.
    pub fn is_whitespace(c: char) -> bool {
        match c {
            ' ' | '\r' | '\t' | '\n' => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ScanError {
    UnterminatedString(SourcePosition),
    UnexpectedCharacter(char, SourcePosition),
}

type Result<T> = std::result::Result<T, ScanError>;

pub struct Scanner<'a> {
    source: MultiPeek<Chars<'a>>,
    lexeme: String,
    position: SourcePosition,
}

// TODO: extracting source iterator
impl<'a> Scanner<'a> {
    // TODO: make Scanner not to be owned
    pub fn new(src: &'a str) -> Self {
        Self {
            source: multipeek(src.chars()),
            lexeme: String::new(),
            position: SourcePosition::initial(),
        }
    }

    pub fn scan(&mut self) -> (Vec<SourceToken>, Vec<ScanError>) {
        let mut tokens = Vec::<SourceToken>::new();
        let mut errors = Vec::<ScanError>::new();

        loop {
            let position = self.position;
            // None is just discarded
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

    fn peek(&mut self) -> Option<&char> {
        self.source.reset_peek();
        self.source.peek()
    }

    fn peek_next(&mut self) -> Option<&char> {
        self.source.peek()
    }

    /// Advances the char-based iterator, incrementing the current position.
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

    /// Returns true if it advanced.
    fn advance_if_match(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Returns hit EoF or not.
    fn advance_until_match(&mut self, expected: char) -> bool {
        loop {
            match self.peek() {
                Some(&c) if c == expected => {
                    return false;
                }
                None => {
                    return true;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn advance_while(&mut self, conditional: &Fn(char) -> bool) {
        while let Some(&c) = self.peek() {
            if !conditional(c) {
                return;
            }
            self.advance();
        }
    }

    /// Returns None for tokens to be discarded.
    fn scan_token(&mut self) -> Option<Result<Token>> {
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
                    if self.advance_until_match('\n') {
                        // Hit EoF
                        return Some(Ok(Eof));
                    } else {
                        return None;
                    }
                } else {
                    Ok(Slash)
                }
            }
            ' ' | '\r' | '\t' | '\n' => return None,
            '"' => self.scan_string(),
            c if char_ext::is_digit(c) => self.scan_number(),
            c if char_ext::is_alpha(c) => self.scan_identifier(),
            _ => return Some(Err(ScanError::UnexpectedCharacter(c, self.position))),
        };

        return Some(_result);
    }

    // TODO: skipping block comment
    fn scan_operator(&mut self, expected: char, if_true: Token, if_false: Token) -> Token {
        if self.peek() == Some(&expected) {
            self.advance();
            if_true
        } else {
            if_false
        }
    }

    fn scan_string(&mut self) -> Result<Token> {
        loop {
            match self.advance() {
                None => return Err(ScanError::UnterminatedString(self.position)),
                Some('"') => {
                    // return removing both " characters
                    return Ok(Token::String(
                        self.lexeme[1..self.lexeme.len() - 1].to_string(),
                    ));
                }
                _ => {}
            };
        }
    }

    // disabled: a leading or trailing decimal point
    // TODO: enabling comma deliminated numbers
    fn scan_number(&mut self) -> Result<Token> {
        self.advance_while(&char_ext::is_digit);
        if self.peek() == Some(&'.') {
            match self.peek_next() {
                Some(&c) if char_ext::is_digit(c) => {
                    self.advance();
                    self.advance_while(&char_ext::is_digit);
                }
                _ => {}
            }
        }

        let n = self
            .lexeme
            .parse()
            .expect(&format!("scan_number parsing error for {}", self.lexeme));
        return Ok(Token::Number(n));
    }

    /// Scans an identifier or a reserved word.
    fn scan_identifier(&mut self) -> Result<Token> {
        self.advance_while(&char_ext::is_alphanumeric);
        use Token::*;
        Ok(match self.lexeme.as_ref() {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "for" => For,
            "fun" => Fun,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "super" => Super,
            "this" => This,
            "true" => True,
            "var" => Var,
            "while" => While,
            name => Identifier(name.to_string()),
        })
    }
}
