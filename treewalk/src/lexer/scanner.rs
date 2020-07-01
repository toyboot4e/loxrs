//! Scanner, tokenizer or lexer

use crate::lexer::token::{Location, Token, TokenKind};
use std::str::Chars;

// This is a VERY BAD scanner.
// Maybe I should use `ByteSpan` and it comes in bytecode interpreter

mod hidden {
    //! Hides fields of `ScanState`

    use crate::lexer::token::Location;
    use itertools::{multipeek, MultiPeek};
    use std::str::Chars;

    // actually I should use `ByteReader`
    /// Trackable `char` reader
    pub struct CharReader<I>
    where
        I: Iterator<Item = char>,
    {
        src: MultiPeek<I>,
        pos: Location,
        lexeme: String,
    }

    impl<'a> CharReader<Chars<'a>> {
        pub fn new(s: &'a str) -> Self {
            CharReader {
                src: multipeek(s.chars()),
                pos: Location::initial(),
                lexeme: String::new(),
            }
        }
    }

    impl<I> Iterator for CharReader<I>
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
                        self.pos.inc_ln();
                        self.pos.init_col();
                    }
                    _ => {
                        self.pos.inc_col();
                    }
                };
            }
            next
        }
    }

    impl<I> CharReader<I>
    where
        I: Iterator<Item = char>,
    {
        pub fn pos(&self) -> Location {
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
    impl<I> CharReader<I>
    where
        I: Iterator<Item = char>,
    {
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
    UnterminatedString(Location),
    UnterminatedRangeComment(Location),
    UnexpectedEof(Location),
    UnexpectedCharacter(char, Location),
}

pub struct Scanner<'a> {
    chars: self::hidden::CharReader<Chars<'a>>,
}

/// Scanner implementation
impl<'a> Scanner<'a> {
    // TODO: make Scanner not to be owned
    pub fn new(src: &'a str) -> Self {
        Self {
            chars: self::hidden::CharReader::new(src),
        }
    }

    fn add_context(&mut self, token: TokenKind, pos: Location) -> Token {
        Token::new(token, pos, self.chars.lexeme().to_string())
    }

    /// Tokenizes a string
    pub fn scan(&mut self) -> (Vec<Token>, Vec<ScanError>) {
        let mut tks = Vec::<Token>::new();
        let mut errs = Vec::<ScanError>::new();
        loop {
            let pos = self.chars.pos();
            match self.next_token() {
                Ok(Some(tk)) => {
                    tks.push(self.add_context(tk, pos));
                }
                Ok(None) => {
                    // EoF
                    break;
                }
                Err(why) => {
                    errs.push(why);
                }
            }
        }

        return (tks, errs);
    }

    fn next_token(&mut self) -> Result<Option<TokenKind>> {
        loop {
            self.chars.clear_lexeme();

            let c = match self.chars.next() {
                None => return Ok(None),
                Some(x) => x,
            };

            use TokenKind::*;
            return Ok(Some(match c {
                // single character token
                '(' => LeftParen,
                ')' => RightParen,
                '{' => LeftBrace,
                '}' => RightBrace,
                ',' => Comma,
                '.' => Dot,
                '+' => Plus,
                '-' => Minus,
                ';' => Semicolon,
                '*' => Star,
                '@' => Self_,

                // comparison
                '!' => self.scan_cmp('=', BangEq, Bang)?,
                '=' => self.scan_cmp('=', EqEq, Eq)?,
                '<' => self.scan_cmp('=', LessEq, Less)?,
                '>' => self.scan_cmp('=', GreaterEq, Greater)?,

                // commenting or division
                '/' => {
                    self.scan_slash()?;
                    continue;
                }

                // logic
                '|' => self.scan_logic('|', Or)?,
                '&' => self.scan_logic('&', And)?,

                // whitespace
                ' ' | '\r' | '\t' | '\n' => continue,

                // literals
                '"' => self.scan_string()?,
                c if char_ext::is_digit(c) => self.scan_number()?,
                c if char_ext::is_alpha(c) => self.scan_kwd_or_ident()?,

                _ => return Err(ScanError::UnexpectedCharacter(c, self.chars.pos())),
            }));
        }
    }

    fn scan_cmp(
        &mut self,
        expected: char,
        if_true: TokenKind,
        if_false: TokenKind,
    ) -> Result<TokenKind> {
        self.chars
            .peek()
            .map(|c| c.clone())
            .map(|c| {
                if c == expected {
                    self.chars.next();
                    if_true
                } else {
                    if_false
                }
            })
            .ok_or_else(|| ScanError::UnexpectedEof(self.chars.pos()))
    }

    /// Expect one `char` and then return the `TokenKind` for it
    fn scan_logic(&mut self, expected: char, if_true: TokenKind) -> Result<TokenKind> {
        match self.chars.next() {
            Some(c) if c == expected => Ok(if_true),
            Some(c) => Err(ScanError::UnexpectedCharacter(c, self.chars.pos())),
            None => Err(ScanError::UnexpectedEof(self.chars.pos())),
        }
    }

    /// slash (`Ok(TokenKind::Slash)`), comment (`Ok(None)`) or `Err`
    fn scan_slash(&mut self) -> Result<Option<TokenKind>> {
        if self.chars.consume_char('/') {
            self.chars.advance_until(|c| c == '\n');
            Ok(None)
        } else if self.chars.consume_char('*') {
            self.scan_range_comment().map(|_| None)
        } else {
            Ok(Some(TokenKind::Slash))
        }
    }

    fn scan_range_comment(&mut self) -> Result<()> {
        // TODO: consider escape
        while let Some(c) = self.chars.next() {
            if c == '*' {
                if self.chars.consume_char('/') {
                    return Ok(());
                }
            }
            // nestable
            if c == '/' {
                if self.chars.consume_char('*') {
                    self.scan_range_comment()?;
                }
            }
        }
        Err(ScanError::UnterminatedRangeComment(self.chars.pos()))
    }

    // TODO: enable rich enclosure such as r#"raw_string"#
    // TODO: enable escapes
    fn scan_string(&mut self) -> Result<TokenKind> {
        loop {
            match self.chars.next() {
                None => return Err(ScanError::UnterminatedString(self.chars.pos())),
                Some('"') => {
                    // strip " characters
                    return Ok(TokenKind::Str(
                        self.chars.lexeme()[1..self.chars.lexeme().len() - 1].to_string(),
                    ));
                }
                _ => {}
            };
        }
    }

    // a leading or trailing decimal point is disabled
    // TODO: enabling comma or underline deliminated numbers
    fn scan_number(&mut self) -> Result<TokenKind> {
        self.chars.advance_while(&char_ext::is_digit);
        if self.chars.peek() == Some(&'.') {
            match self.chars.peek_next() {
                Some(&c) if char_ext::is_digit(c) => {
                    self.chars.next();
                    self.chars.advance_while(&char_ext::is_digit);
                }
                _ => {}
            }
        }

        let n = self.chars.lexeme().parse().expect(&format!(
            "scan_number parsing error for {}",
            self.chars.lexeme()
        ));
        return Ok(TokenKind::Num(n));
    }

    /// Scans an identifier or a reserved word.
    fn scan_kwd_or_ident(&mut self) -> Result<TokenKind> {
        self.chars.advance_while(&char_ext::is_alphanumeric);
        use TokenKind::*;
        Ok(match self.chars.lexeme().as_ref() {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "for" => For,
            "fn" => Fn,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "super" => Super,
            "true" => True,
            "var" => Var,
            "while" => While,
            name => Ident(name.to_string()),
        })
    }
}
