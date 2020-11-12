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
        let next = self.peek0()?.clone();
        self.sp.hi.0 += 1;
        Some(next)
    }

    /// Returns the number of bytes that matches to the predicate
    pub fn peek_while(&mut self, offset: usize, p: &mut impl FnMut(u8) -> bool) -> usize {
        let mut ix = offset;
        while let Some(b) = self.peek_n(ix) {
            if !p(b) {
                break;
            } else {
                ix += 1;
            }
        }
        ix - offset
    }

    /// * `n`: the number of bytes to skip
    pub fn skip_n(&mut self, n: usize) {
        self.sp.hi.0 += n;
    }

    pub fn peek_n(&self, offset: usize) -> Option<u8> {
        let ix = self.sp.hi.0 + offset;
        if ix < self.src.len() {
            Some(self.src[ix])
        } else {
            None
        }
    }

    pub fn peek0(&mut self) -> Option<u8> {
        self.peek_n(0)
    }

    pub fn peek1(&mut self) -> Option<u8> {
        self.peek_n(1)
    }

    pub fn consume_skipped(&mut self) -> ByteSpan {
        let sp = self.sp;
        self.sp.lo = self.sp.hi;
        sp
    }

    pub fn consume_skipping(&mut self, len: usize) -> ByteSpan {
        self.skip_n(len);
        self.consume_skipped()
    }
}

/// Parsers
impl<'a> LexState<'a> {
    // scanner functions
    pub fn ws(&mut self) -> Result<Option<SpanToken>> {
        let prev_hi = self.sp.hi;

        fn is_ws(b: u8) -> bool {
            matches!(b, b' ' | b'\r' | b'\t')
        }

        while let Some(c) = self.peek0() {
            match c {
                b if is_ws(b) => {
                    self.skip_n(1);
                }

                b'/' => match self.peek1() {
                    Some(c) if c == b'/' => {
                        return Ok(Some(self.line_comment()));
                    }
                    Some(c) if c == b'*' => {
                        return Ok(Some(self.range_comment()?));
                    }
                    _ => {
                        break;
                    }
                },

                _ => break,
            };
        }

        if self.sp.hi != prev_hi {
            Ok(Some(SpanToken::new(Token::Ws, self.consume_skipped())))
        } else {
            Ok(None)
        }
    }

    fn line_comment(&mut self) -> SpanToken {
        self.skip_n(2);

        while let Some(c) = self.next() {
            if c == b'\n' {
                break;
            }
        }

        SpanToken::new(Token::LineComment, self.consume_skipped())
    }

    fn range_comment(&mut self) -> Result<SpanToken> {
        self.skip_n(2);

        while let Some(c) = self.next() {
            match c {
                b'*' if matches!(self.peek0(), Some(b'/')) => {
                    return Ok(SpanToken::new(Token::RangeComment, self.consume_skipped()));
                }
                b'/' if matches!(self.peek0(), Some(b'*')) => {
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

        if self.peek_n(len_whole) != Some(b'.') {
            self.skip_n(len_whole);
            return Ok(Some(SpanToken::new(Token::Num, self.consume_skipped())));
        }

        let len_decimal = self.peek_while(len_whole + 1, &mut |b| matches!(b, b'0'..=b'9'));

        self.skip_n(len_whole + 1 + len_decimal);
        Ok(Some(SpanToken::new(Token::Num, self.consume_skipped())))
    }

    pub fn str(&mut self) -> Result<Option<SpanToken>> {
        // "
        if self.peek0() != Some(b'"') {
            return Ok(None);
        }

        // TODO: should I return `Token::StrStart` here?
        self.skip_n(1);

        fn try_peek_n(me: &mut LexState, offset: usize) -> Result<u8> {
            me.peek_n(offset)
                .ok_or_else(|| LexError::UnterminatedString {
                    start: me.consume_skipping(offset).lo,
                })
        }

        // <content> "
        let mut len_content = 0;
        loop {
            let b = try_peek_n(self, len_content)?;

            match self.peek_n(len_content) {
                // escape
                Some(b) if b == b'\\' => {
                    //
                }
                // terminated
                Some(b) if b == b'"' => {
                    break;
                }
                // normal byte
                Some(_) => len_content += 1,
                // EoF
                None => {
                    return Err(LexError::UnterminatedString {
                        start: self.consume_skipping(len_content).lo,
                    });
                }
            }
        }

        self.skip_n(len_content);
        self.skip_n(1); // "

        Ok(Some(SpanToken::new(Token::Str, self.consume_skipped())))
    }

    /// [a-zA-Z_][a-zA-Z0-9]+
    pub fn kwd(&mut self) -> Option<SpanToken> {
        fn is_in(x: u8, a: u8, b: u8) -> bool {
            !(x < a || b < x)
        }

        fn is_word(b: u8) -> bool {
            is_in(b, b'a', b'z') || is_in(b, b'A', b'Z') || b >= 1 >> 7
        }

        fn is_word_or_num(b: u8) -> bool {
            is_word(b) || is_in(b, b'0', b'9')
        }

        if !is_word(self.peek0()?) {
            return None;
        }

        let len = 1 + self.peek_while(1, &mut is_word_or_num);

        self.skip_n(len);
        let s = &self.src[self.sp.lo.0..self.sp.hi.0];

        unimplemented!()
    }

    pub fn ident(&mut self) -> Result<Option<Token>> {
        unimplemented!("ident");
    }

    pub fn symbol(&mut self) -> Option<SpanToken> {
        let tk = match self.peek0()? {
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

            _b => return None,
        };

        self.skip_n(1);
        Some(SpanToken::new(tk, self.consume_skipped()))
    }

    fn one_two(&mut self, default: Token, expected: u8, if_match: Token) -> Token {
        match self.peek1() {
            None => default,
            Some(n) if n == expected => {
                self.skip_n(1);
                if_match
            }
            _ => default,
        }
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
        if self.state.peek0().is_none() {
            return Ok(SpanToken::new(
                Token::Eof,
                // the length is zero but it's ok
                ByteSpan::new(self.state.lo(), self.state.lo()),
            ));
        }

        // match rules on by one

        if let Some(stk) = self.state.ws()? {
            return Ok(stk);
        }

        if let Some(stk) = self.state.num()? {
            return Ok(stk);
        }

        if let Some(stk) = self.state.str()? {
            return Ok(stk);
        }

        // if let Some(tk) = self.state.kwd_or_ident()? {
        //     return Ok(tk);
        // }

        if let Some(stk) = self.state.symbol() {
            return Ok(stk);
        }

        let b = self.state.peek0().unwrap();
        Err(LexError::UnexpectedByte {
            pos: self.state.hi(),
            byte: b,
        })
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

        assert_eq!(tks, expected, "\nsrc: {}", src);

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

    #[test]
    fn string() -> Result<()> {
        let src = r##" "string" "##;
        let tks = self::run_lexer(src)?;
        let expected = &[
            SpanToken::new(Token::Ws, [0, 1]),
            SpanToken::new(Token::Str, [1, 9]),
            SpanToken::new(Token::Ws, [9, 10]),
        ];
        assert_eq!(tks, expected, "\nsrc: {}", src);
        Ok(())
    }
}
