use thiserror::Error;

use crate::parse::{
    span::{BytePos, ByteSpan},
    token::{SpanToken, Token},
};

pub type Result<T> = std::result::Result<T, LexError>;

/// Byte-based error report
#[derive(Debug, Error)]
pub enum LexError {
    #[error("unterminated string")]
    UnterminatedString { start: BytePos },
    #[error("unterminated commet")]
    UnterminatedComment { start: BytePos },
    #[error("unexpected EoF")]
    UnexpectedEof(),
    #[error("unexpected character")]
    UnexpectedCharacter { found: char, at: BytePos },
    #[error("unexpected token: {found:?}, expected: {expected:?}")]
    UnexpectedToken {
        found: SpanToken,
        expected: Vec<Token>,
    },
    #[error("unexpected byte: {pos:?},{byte}")]
    UnexpectedByte { pos: BytePos, byte: u8 },
}

/// Inner state for implementing [`Lexer`]
struct LexState<'a> {
    src: &'a [u8],
    sp: ByteSpan,
}

impl<'a> LexState<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.as_ref(),
            sp: ByteSpan::default(),
        }
    }

    pub fn lo(&self) -> BytePos {
        self.sp.lo
    }

    pub fn hi(&self) -> BytePos {
        self.sp.hi
    }

    pub fn next(&mut self) -> Option<u8> {
        let next = self.peek1()?.clone();
        self.sp.hi.0 += 1;
        Some(next)
    }

    /// Returns the number of bytes that matches to the predicate
    pub fn peek_while(&mut self, offset: usize, p: &mut impl FnMut(u8) -> bool) -> usize {
        let mut ix = offset;
        while let Some(b) = self.peek_n(ix) {
            if !p(*b) {
                break;
            } else {
                ix += 1;
            }
        }
        ix - offset
    }

    pub fn skip_n(&mut self, n: usize) {
        self.sp.hi.0 += n;
    }

    pub fn peek_n(&self, offset: usize) -> Option<&u8> {
        let ix = self.sp.hi.0 + offset;
        if ix < self.src.len() {
            Some(&self.src[ix])
        } else {
            None
        }
    }

    pub fn peek1(&mut self) -> Option<&u8> {
        self.peek_n(0)
    }

    pub fn peek2(&mut self) -> Option<&u8> {
        self.peek_n(1)
    }

    pub fn consume_span(&mut self) -> ByteSpan {
        let sp = self.sp;
        self.sp.lo = self.sp.hi;
        sp
    }
}

/// Parsers
impl<'a> LexState<'a> {
    // scanner functions
    pub fn ws(&mut self) -> Result<Option<SpanToken>> {
        let prev_hi = self.sp.hi;

        while let Some(c) = self.peek1() {
            match *c {
                b' ' | b'\r' | b'\t' => {
                    self.skip_n(1);
                }

                b'/' => match self.peek2() {
                    Some(c) if *c == b'/' => {
                        return Ok(Some(self.line_comment()));
                    }
                    Some(c) if *c == b'*' => {
                        return Ok(Some(self.range_comment()?));
                    }
                    _ => {
                        break;
                    }
                },

                _ => break,
            };
        }

        Ok(if self.sp.hi != prev_hi {
            Some(SpanToken::new(Token::Ws, self.consume_span()))
        } else {
            None
        })
    }

    fn line_comment(&mut self) -> SpanToken {
        self.skip_n(2);

        while let Some(c) = self.next() {
            if c == b'\n' {
                break;
            }
        }

        SpanToken::new(Token::LineComment, self.consume_span())
    }

    fn range_comment(&mut self) -> Result<SpanToken> {
        self.skip_n(2);

        while let Some(c) = self.next() {
            match c {
                b'*' if matches!(self.peek1(), Some(&b'/')) => {
                    return Ok(SpanToken::new(Token::RangeComment, self.consume_span()));
                }
                b'/' if matches!(self.peek1(), Some(&b'*')) => {
                    self.range_comment()?;
                }
                _ => {}
            }
        }

        Err(LexError::UnterminatedComment { start: self.sp.lo })
    }

    /// [0-9]+ ("." [0-9]+)?
    ///
    /// ```none
    /// 130.456
    /// ^  ^  ^
    /// |  |  + decimal value
    /// |  + decimal point
    /// + whole value
    /// ```
    ///
    /// Allows invalid format
    pub fn num(&mut self) -> Result<Option<SpanToken>> {
        let len_whole = self.peek_while(0, &mut |b| matches!(b, b'0'..=b'9'));

        if len_whole == 0 {
            return Ok(None);
        }

        if self.peek_n(len_whole) != Some(&b'.') {
            self.skip_n(len_whole);
            return Ok(Some(SpanToken::new(Token::Num, self.consume_span())));
        }

        let len_decimal = self.peek_while(len_whole + 1, &mut |b| matches!(b, b'0'..=b'9'));

        self.skip_n(len_whole + 1 + len_decimal);
        Ok(Some(SpanToken::new(Token::Num, self.consume_span())))
    }

    pub fn str(&mut self) -> Result<Option<Token>> {
        unimplemented!("str");
    }

    pub fn ident(&mut self) -> Result<Option<Token>> {
        unimplemented!("ident");
    }
}

/// A tokenizer of `Iterator<Item = char>`
pub struct Lexer<'a> {
    state: LexState<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            state: LexState::new(src),
        }
    }

    /// Next [`SpanToken`]
    pub fn next_stk(&mut self) -> Result<SpanToken> {
        // match rules on by one

        if let Some(stk) = self.state.ws()? {
            return Ok(stk);
        }

        if let Some(tk) = self.state.num()? {
            return Ok(tk);
        }

        // if let Some(tk) = self.state.ident()? {
        //     return Ok(tk);
        // }

        let tk = self.next_tk()?;
        let sp = self.state.consume_span();

        Ok(SpanToken::new(tk, sp))
    }

    fn next_tk(&mut self) -> Result<Token> {
        let c = match self.state.next() {
            None => return Ok(Token::Eof),
            Some(c) => c,
        };

        Ok(match c {
            b'(' => Token::LParen,
            b')' => Token::RParen,
            b'{' => Token::LBrace,
            b'}' => Token::RBrace,
            b':' => Token::Colon,
            b';' => Token::Semicolon,
            b',' => Token::Comma,
            b'.' => Token::Dot,
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'*' => Token::Star,
            b'/' => Token::Slash,
            b'!' => self.one_two(Token::Bang, b'=', Token::BangEq),
            b'=' => self.one_two(Token::Eq, b'=', Token::EqEq),
            b'<' => self.one_two(Token::Lt, b'=', Token::Le),
            b'>' => self.one_two(Token::Gt, b'=', Token::Ge),

            b => {
                return Err(LexError::UnexpectedByte {
                    pos: self.state.lo(),
                    byte: b,
                });
            }
        })
    }

    fn one_two(&mut self, not_match: Token, expected: u8, if_match: Token) -> Token {
        match self.state.peek1() {
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
    use crate::parse::{
        lexer::{Lexer, Result},
        token::{SpanToken, Token},
    };

    fn run_lexer(src: &str) -> Result<Vec<SpanToken>> {
        let mut lex = Lexer::new(src);
        let mut tks = Vec::with_capacity(50);

        loop {
            let stk = lex.next_stk()?;
            if stk.tk == Token::Eof {
                break;
            }
            tks.push(stk);
        }

        Ok(tks)
    }

    fn match_tokens(src: &str, expected: &[Token]) -> Result<()> {
        let tks = self::run_lexer(src)?
            .into_iter()
            .map(|stk| stk.tk)
            .collect::<Vec<_>>();

        assert_eq!(tks, expected);

        Ok(())
    }

    #[test]
    fn spans() -> Result<()> {
        let src = ":  , ;";
        let tks = self::run_lexer(src)?;
        let expected = &[
            SpanToken::new(Token::Colon, [0, 1]),
            SpanToken::new(Token::Ws, [1, 3]),
            SpanToken::new(Token::Comma, [3, 4]),
            SpanToken::new(Token::Ws, [4, 5]),
            SpanToken::new(Token::Semicolon, [5, 6]),
        ];
        assert_eq!(tks, expected, "\nsrc: {}", src);
        Ok(())
    }

    #[test]
    fn just_symbols() -> Result<()> {
        self::match_tokens(
            "(){};,. +-*/ <><=>= ===!!=",
            &[
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
                Token::Semicolon,
                Token::Comma,
                Token::Dot,
                //
                Token::Ws,
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                //
                Token::Ws,
                Token::Lt,
                Token::Gt,
                Token::Le,
                Token::Ge,
                //
                Token::Ws,
                Token::EqEq,
                Token::Eq,
                Token::Bang,
                Token::BangEq,
            ],
        )
    }

    #[test]
    fn number() -> Result<()> {
        let src = "3.14 * 0.25";
        let tks = self::run_lexer(src)?;
        let expected = &[
            SpanToken::new(Token::Num, [0, 4]),
            SpanToken::new(Token::Ws, [4, 5]),
            SpanToken::new(Token::Star, [5, 6]),
            SpanToken::new(Token::Ws, [6, 7]),
            SpanToken::new(Token::Num, [7, 11]),
        ];
        assert_eq!(tks, expected, "\nsrc: {}", src);
        Ok(())
    }
}
