use crate::abs::expr::*;
use crate::token::{SourcePosition, SourceToken, Token};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum ParseError {
    Temp, // Delete this with suitable error
    UnexpectedEof,
}
type Result = std::result::Result<Expr, ParseError>;

pub struct Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    tokens: Peekable<I>,
}

impl<'a> Parser<'a, std::slice::Iter<'a, SourceToken>> {
    // TODO: more abstarct constructor
    // (maybe allowing implicit type conversion or like that)
    pub fn new(tokens: &'a [SourceToken]) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    fn peek(&mut self) -> Option<&&SourceToken> {
        self.tokens.peek()
    }

    /// True if any matches to the next token.
    fn advance_and_match(&mut self, tokens: &[Token]) -> Option<Token> {
        let next = match self.advance() {
            None => return None,
            Some(source_token) => &source_token.token,
        };
        return tokens.iter().find(|t| t == &next).map(|t| t.clone());
    }

    fn advance(&mut self) -> Option<&SourceToken> {
        self.tokens.next()
    }

    pub fn parse_expr(&mut self) -> Result {
        self.parse_equality()
    }

    /// Right recursive parsing
    // TODO: #[inline], meta programming, etc.
    fn rrp<Oper>(
        &mut self,
        sub_rule: &Fn(&mut Self) -> Result,
        delimiters: &[Token],
        folder: &Fn(Expr, Oper, Expr) -> Expr,
    ) -> Result
    where
        Token: Into<Option<Oper>>, // where
                                   //     Token: Into<Option<Oper>>
    {
        let mut expr = sub_rule(self)?;
        while let Some(token) = self.advance_and_match(delimiters) {
            let right = sub_rule(self)?;
            expr = folder(expr, token.into().unwrap(), right);
        }
        return Ok(expr);
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn parse_equality(&mut self) -> Result {
        use Token::*;
        self.rrp::<BinaryOper>(
            &Self::parse_comparison,
            &[EqualEqual, BangEqual],
            &Expr::binary,
        )
    }

    /// comparison → addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
    fn parse_comparison(&mut self) -> Result {
        use Token::*;
        self.rrp(
            &Self::parse_addition,
            &[Greater, GreaterEqual, Less, LessEqual],
            &Expr::binary,
        )
    }

    /// addition → multiplication ( ( "-" | "+" ) multiplication )* ;
    fn parse_addition(&mut self) -> Result {
        use Token::*;
        self.rrp(&Self::parse_multiplication, &[Plus, Minus], &Expr::binary)
    }

    /// multiplication → unary ( ( "/" | "*" ) unary )* ;
    fn parse_multiplication(&mut self) -> Result {
        use Token::*;
        self.rrp(&Self::parse_unary, &[Slash, Star], &Expr::binary)
    }

    /// unary   → ( "!" | "-" ) unary | primary ;
    fn parse_unary(&mut self) -> Result {
        use Token::*;
        self.rrp(&Self::parse_primary, &[Bang, Minus], &Expr::binary)
    }

    /// primary → NUMBER | STRING | "false" | "true" | "nil" | "(" expression ")" ;
    fn parse_primary(&mut self) -> Result {
        let token = match self.advance() {
            Some(s_token) => &s_token.token,
            None => return Err(ParseError::UnexpectedEof),
        };
        use Token::*;
        Ok(match *token {
            // literal or grouping
            Number(n) => LiteralArgs::Number(n),
            String(ref s) => LiteralArgs::StringL(s.clone()),
            False => LiteralArgs::Bool(false),
            True => LiteralArgs::Bool(true),
            Nil => LiteralArgs::Nil,
            _ => return self.parse_group(),
        }
        .into())
    }

    /// "(" expression ")"
    /// To be called after consuming "("
    fn parse_group(&mut self) -> Result {
        return Err(ParseError::Temp);
    }

    /// Enters panic mode and tries to go to next statement (though it's not so accurate).
    /// It goes to a next semicolon, which may not be the beginning of the next statement.
    fn synchronize(&mut self) {
        while let Some(s_token) = self.peek() {
            let result = SyncPeekChecker::check_token(&s_token.token);
            if result.needs_advance {
                self.advance();
            }
            if result.ends {
                break;
            }
        }
    }
}

struct SyncPeekChecker {
    pub needs_advance: bool,
    pub ends: bool,
}

use std::borrow::Borrow;
impl SyncPeekChecker {
    // TODO: proper Borrow<Token> or just use &Token
    pub fn check_token<T: Borrow<Token>>(token: T) -> Self {
        use Token::*;
        match token.borrow() {
            Class | Fun | Var | If | For | While | Print | Return => Self {
                needs_advance: false,
                ends: true,
            },
            Semicolon => Self {
                needs_advance: true,
                ends: true,
            },
            _ => Self {
                needs_advance: true,
                ends: false,
            },
        }
    }
}
