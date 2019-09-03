use crate::abs::token::{SourcePosition, SourceToken, Token};
use std::str::Chars;

/// Hides fields of `ScanState`
mod hidden {
    use ::itertools::{multipeek, MultiPeek};

    use crate::abs::token::SourcePosition;
    use std::str::Chars;

    pub struct ScanState<I>
    where
        I: Iterator<Item = char>,
    {
        src: MultiPeek<I>,
        pos: SourcePosition,
        lexeme: String,
    }

    impl<'a> ScanState<Chars<'a>> {
        pub fn new(s: &'a str) -> Self {
            ScanState {
                src: multipeek(s.chars()),
                pos: SourcePosition::initial(),
                lexeme: String::new(),
            }
        }
    }

    impl<I> Iterator for ScanState<I>
    where
        I: Iterator<Item = char>,
    {
        type Item = char;
        fn next(&mut self) -> Option<char> {
            let next = self.src.next();
            if let Some(c) = next {
                self.lexeme.push(c);
                match c {
                    '\n' => {
                        self.pos.inc_line();
                        self.pos.init_column();
                    }
                    _ => {
                        self.pos.inc_column();
                    }
                };
            }
            next
        }
    }

    impl<I> ScanState<I>
    where
        I: Iterator<Item = char>,
    {
        pub fn pos(&self) -> SourcePosition {
            self.pos
        }

        pub fn lexeme(&self) -> &str {
            &self.lexeme
        }

        pub fn peek(&mut self) -> Option<&char> {
            self.src.reset_peek();
            self.src.peek()
        }

        pub fn peek_next(&mut self) -> Option<&char> {
            self.src.peek()
        }

        pub fn clear_lexeme(&mut self) {
            self.lexeme.clear();
        }
    }

    /// Mutation-based iterational methods
    impl<I> ScanState<I>
    where
        I: Iterator<Item = char>,
    {
        pub fn next_if<P>(&mut self, predicate: P) -> Option<char>
        where
            P: Fn(&char) -> bool,
        {
            if let Some(c) = self.peek() {
                if predicate(c) {
                    self.next()
                } else {
                    None
                }
            } else {
                None
            }
        }

        // TODO: char vs &char
        /// Advances if the next character is `c`
        pub fn consume_char(&mut self, c: char) -> bool {
            if Some(&c) == self.peek() {
                self.next();
                true
            } else {
                false
            }
        }

        /// Advances while the peek matches `predicate`; peeks char by char
        pub fn advance_while<P>(&mut self, predicate: P) -> bool
        where
            P: Fn(char) -> bool,
        {
            while let Some(&c) = self.peek() {
                if !predicate(c) {
                    return true;
                }
                self.next();
            }
            return false;
        }

        /// Advances until finding; doesn't peek
        pub fn advance_until<P>(&mut self, predicate: P) -> bool
        where
            P: Fn(char) -> bool,
        {
            while let Some(c) = self.next() {
                if predicate(c) {
                    return true;
                }
            }
            return false;
        }
    }
}

mod char_ext {
    pub fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    pub fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    pub fn is_alphanumeric(c: char) -> bool {
        is_digit(c) || is_alpha(c)
    }
}

type Result<T> = std::result::Result<T, ScanError>;
#[derive(Debug, Clone)]
pub enum ScanError {
    UnterminatedString(SourcePosition),
    UnexpectedEof(SourcePosition),
    UnexpectedCharacter(char, SourcePosition),
}

pub struct Scanner<'a> {
    state: hidden::ScanState<Chars<'a>>,
}

/// Scanner implementation
impl<'a> Scanner<'a> {
    // TODO: make Scanner not to be owned
    pub fn new(src: &'a str) -> Self {
        Self {
            state: hidden::ScanState::new(src),
        }
    }

    fn add_context(&mut self, token: Token, pos: SourcePosition) -> SourceToken {
        SourceToken::new(token, pos, self.state.lexeme().to_string())
    }

    pub fn scan(&mut self) -> (Vec<SourceToken>, Vec<ScanError>) {
        let mut tokens = Vec::<SourceToken>::new();
        let mut errors = Vec::<ScanError>::new();
        loop {
            let pos = self.state.pos();
            if let Some(result) = self.scan_token() {
                match result {
                    Ok(Token::Eof) => {
                        break;
                    }
                    Ok(token) => {
                        tokens.push(self.add_context(token, pos));
                    }
                    Err(why) => {
                        errors.push(why);
                    }
                }
            };
        }

        return (tokens, errors);
    }

    /// Returns None for tokens to be discarded.
    fn scan_token(&mut self) -> Option<Result<Token>> {
        use Token::*;
        self.state.clear_lexeme();

        let c = match self.state.next() {
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
            '!' => self.scan_operator('=', BangEqual, Bang),
            '=' => self.scan_operator('=', EqualEqual, Equal),
            '<' => self.scan_operator('=', LessEqual, Less),
            '>' => self.scan_operator('=', GreaterEqual, Greater),
            '/' => {
                if self.state.consume_char('/') {
                    self.state.advance_until(|c| c == '\n');
                    return if self.state.peek().is_some() {
                        None
                    } else {
                        Some(Ok(Eof))
                    };
                } else {
                    Ok(Slash)
                }
            }
            // skip_while
            ' ' | '\r' | '\t' | '\n' => return None,
            '"' => self.scan_string(),
            c if char_ext::is_digit(c) => self.scan_number(),
            c if char_ext::is_alpha(c) => self.scan_identifier(),
            _ => Err(ScanError::UnexpectedCharacter(c, self.state.pos())),
        };

        return Some(_result);
    }

    fn scan_operator(&mut self, expected: char, if_true: Token, if_false: Token) -> Result<Token> {
        self.state
            .next()
            .map(|c| if c == expected { if_true } else { if_false })
            .ok_or_else(|| ScanError::UnexpectedEof(self.state.pos()))
    }

    // TODO: enable rich enclosure such as ###"
    // TODO: enable escapes
    fn scan_string(&mut self) -> Result<Token> {
        loop {
            match self.state.next() {
                None => return Err(ScanError::UnterminatedString(self.state.pos())),
                Some('"') => {
                    // remove both " characters
                    return Ok(Token::String(
                        self.state.lexeme()[1..self.state.lexeme().len() - 1].to_string(),
                    ));
                }
                _ => {}
            };
        }
    }

    // disabled: a leading or trailing decimal point
    // TODO: enabling comma deliminated numbers
    fn scan_number(&mut self) -> Result<Token> {
        self.state.advance_while(&char_ext::is_digit);
        if self.state.peek() == Some(&'.') {
            match self.state.peek_next() {
                Some(&c) if char_ext::is_digit(c) => {
                    self.state.next();
                    self.state.advance_while(&char_ext::is_digit);
                }
                _ => {}
            }
        }

        let n = self.state.lexeme().parse().expect(&format!(
            "scan_number parsing error for {}",
            self.state.lexeme()
        ));
        return Ok(Token::Number(n));
    }

    /// Scans an identifier or a reserved word.
    fn scan_identifier(&mut self) -> Result<Token> {
        self.state.advance_while(&char_ext::is_alphanumeric);
        use Token::*;
        Ok(match self.state.lexeme().as_ref() {
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
