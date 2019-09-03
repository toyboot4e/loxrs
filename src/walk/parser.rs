use crate::abs::{expr::*, stmt::*, token::*};
use std::iter::Peekable;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    // TODO: EoF error
    UnexpectedEof,
    UnexpectedToken(UnexpectedTokenErrorArgs),
}

impl ParseError {
    pub fn token(found: &SourceToken, expected: &[Token]) -> Self {
        ParseError::UnexpectedToken(UnexpectedTokenErrorArgs::from_s_token(found, expected))
    }

    pub fn not_beginning_of_stmt(found: &SourceToken) -> Self {
        use Token::*;
        Self::token(found, &[Print])
    }

    pub fn eof() -> Self {
        ParseError::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct UnexpectedTokenErrorArgs {
    pos: SourcePosition,
    expected: Vec<Token>,
    found: Token,
}

impl UnexpectedTokenErrorArgs {
    // TODO: more generic interface
    pub fn from_s_token(s_token: &SourceToken, expected: &[Token]) -> Self {
        UnexpectedTokenErrorArgs {
            pos: s_token.pos,
            expected: expected.iter().cloned().collect(),
            found: s_token.token.clone(),
        }
    }
}

pub struct Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    tokens: Peekable<I>,
}

impl<'a> Parser<'a, std::slice::Iter<'a, SourceToken>> {
    // TODO: more abstarct constructor
    pub fn new(tokens: &'a [SourceToken]) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }
}

/// Iterator implementations
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    fn peek(&mut self) -> Option<&&SourceToken> {
        self.tokens.peek()
    }

    fn next(&mut self) -> Option<&SourceToken> {
        self.tokens.next()
    }

    fn advance(&mut self) -> bool {
        self.tokens.next().is_some()
    }

    fn try_peek(&mut self) -> Result<&&SourceToken> {
        self.peek().ok_or(ParseError::eof())
    }

    fn try_advance(&mut self) -> Result<&SourceToken> {
        self.next().ok_or(ParseError::eof())
    }

    /// Just a wrapper around `Iterator::find`.
    fn _find(s_token: &SourceToken, expected: &[Token]) -> Option<Token> {
        expected.iter().cloned().find(|t| t == &s_token.token)
    }

    /// Just a wrapper around `Iterator::any`.
    fn _any(s_token: &SourceToken, expected: &[Token]) -> bool {
        expected.iter().any(|t| t == &s_token.token)
    }

    /// Returns some matched token or None. Just a wrapper around `Parser::peek` and
    /// `Iterator::any`.
    /// Copies a `Token` if it's found.
    fn peek_find(&mut self, expected: &[Token]) -> Option<&&SourceToken> {
        let s_token = self.peek()?;
        if Self::_any(s_token, expected) {
            Some(s_token)
        } else {
            None
        }
    }

    fn peek_any(&mut self, expected: &[Token]) -> Option<bool> {
        Some(Self::_any(self.peek()?, expected))
    }

    /// Consumes the next token if it matches any of the expected tokens.
    /// Returns None if at EoF.
    fn advance_if_any(&mut self, expected: &[Token]) -> Option<bool> {
        let opt = self.peek_any(expected);
        if let Some(true) = opt {
            self.next();
        }
        opt
    }

    fn advance_if_find(&mut self, expected: &[Token]) -> Option<Token> {
        let opt = Self::_find(self.peek()?, expected);
        if opt.is_some() {
            self.next();
        }
        opt
    }

    /// Just a wrapper of `Iterator::any`. Fails if nothing is found.
    /// Copies a `Token` if it's found.
    fn _try_find(s_token: &SourceToken, expected: &[Token]) -> Result<Token> {
        Self::_find(s_token, expected).ok_or(ParseError::token(s_token, expected))
    }

    /// Fails if the peek doesn't match any of expected tokens.
    /// Copies a `Token` if it's found.
    fn try_peek_find(&mut self, expected: &[Token]) -> Result<Token> {
        self.try_peek()
            .and_then(|s_token| Self::_try_find(s_token, expected))
    }

    /// Calls `Self::try_peek_find` and advance if it's ok.
    /// Copies a`Token` if it's found.
    fn try_advance_if_find(&mut self, expected: &[Token]) -> Result<Token> {
        let result = self.try_peek_find(expected);
        if result.is_ok() {
            self.next();
        }
        result
    }
}

/// Statement Parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    /// program → declaration* EOF ;
    ///
    /// The entry point of the predictive parsing.
    pub fn parse(&mut self) -> (Vec<Stmt>, Vec<ParseError>) {
        let mut stmts = Vec::<Stmt>::new();
        let mut errors = Vec::<ParseError>::new();

        while let Some(s_token) = self.parse_any() {
            match s_token {
                Ok(stmt) => stmts.push(stmt),
                Err(why) => {
                    errors.push(why);
                    self.synchronize();
                }
            }
        }

        return (stmts, errors);
    }

    /// Enters "panic mode" and tries to go to next statement.
    ///
    /// It goes to a next semicolon.
    fn synchronize(&mut self) {
        while let Some(s_token) = self.peek() {
            let result = SyncPeekChecker::check_token(&s_token.token);
            if result.needs_advance {
                self.next();
            }
            if result.ends {
                break;
            }
        }
    }

    /// root → declaration
    /// declaration → varDecl | statement ;
    ///
    /// The root of predictive parsing. Sub rules are named as `stmt_xxx` or `stmt_expr`.
    fn parse_any(&mut self) -> Option<Result<Stmt>> {
        if self.advance_if_any(&[Token::Var])? {
            self.stmt_var().into()
        } else {
            self.parse_stmt().into()
        }
    }

    /// varDecl → "var" IDENTIFIER "=" expression ";" ;
    ///
    /// The initializer is always needed, different from the original Lox.
    fn stmt_var(&mut self) -> Result<Stmt> {
        let s_token = self.try_peek()?;
        if let Token::Identifier(ref name) = s_token.token {
            let name = name.clone(); // releases &mut self
            self.next(); // consuming the identifier
            self.try_advance_if_find(&[Token::Equal])?;
            let init = self.parse_expr()?;
            self.try_advance_if_find(&[Token::Semicolon])?;
            Ok(Stmt::var_dec(name, init))
        } else {
            Err(ParseError::token(s_token, &[Token::Identifier("".into())]))
        }
    }

    /// statement → exprStmt | printStmt ;
    ///
    /// Note that sub rules don't consume unexpected tokens.
    pub fn parse_stmt(&mut self) -> Result<Stmt> {
        use Token::*;
        match &self.try_peek()?.token {
            Print => {
                self.next();
                self.stmt_print()
            }
            _ => self.stmt_expr(),
        }
    }

    /// printStmt → "print" expression ";" ;
    ///
    /// To be called after consuming `print` (predictive parsing).
    fn stmt_print(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;
        self.try_advance_if_find(&[Token::Semicolon])?;
        // TODO: adding Expr -> String functions for printing
        Ok(Stmt::print(expr))
    }

    fn stmt_expr(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;
        self.try_advance_if_find(&[Token::Semicolon])?;
        Ok(Stmt::expr(expr))
    }
}

// Expression parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    /// Rule → Expr (Oper Expr)*
    ///
    /// Abstracts right recursive parsing.
    #[inline]
    fn rrp<Oper, SubRule, Folder>(
        &mut self,
        sub_rule: SubRule,
        delimiters: &[Token],
        folder: Folder,
    ) -> Result<Expr>
    where
        Token: Into<Option<Oper>>,
        SubRule: Fn(&mut Self) -> Result<Expr>,
        Folder: Fn(Expr, Oper, Expr) -> Expr,
    {
        let mut expr = sub_rule(self)?;
        while let Some(token) = self.advance_if_find(delimiters) {
            let right = sub_rule(self)?;
            let oper = token.into().unwrap();
            expr = folder(expr, oper, right);
        }
        return Ok(expr);
    }

    /// Note: doesn't consume semicolon.
    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.expr_equality()
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn expr_equality(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(
            &Self::expr_comparison,
            &[EqualEqual, BangEqual],
            &Expr::binary,
        )
    }

    /// comparison → addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
    fn expr_comparison(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(
            &Self::expr_addition,
            &[Greater, GreaterEqual, Less, LessEqual],
            &Expr::binary,
        )
    }

    /// addition → multiplication ( ( "-" | "+" ) multiplication )* ;
    fn expr_addition(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_multiplication, &[Plus, Minus], &Expr::binary)
    }

    /// multiplication → unary ( ( "/" | "*" ) unary )* ;
    fn expr_multiplication(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_unary, &[Slash, Star], &Expr::binary)
    }

    /// unary → ( "!" | "-" ) unary | primary ;
    fn expr_unary(&mut self) -> Result<Expr> {
        use Token::*;
        match self.try_peek()?.token {
            Minus => Ok(Expr::unary(UnaryOper::Minus, self.expr_unary()?)),
            Bang => Ok(Expr::unary(UnaryOper::Not, self.expr_unary()?)),
            _ => self.expr_primary(),
        }
    }

    /// primary → literal | group | indentifier ;
    ///
    /// literal → number | string | "false" | "true" | "nil" ;
    /// group   → "(" expression ")" ;
    ///
    /// Make sure that there exists next token (predictive parsing).
    fn expr_primary(&mut self) -> Result<Expr> {
        let s_token = self.next().unwrap();
        if let Some(args) = LiteralArgs::from_token(&s_token.token) {
            return Ok(args.into());
        }
        use Token::*;
        match s_token.token {
            LeftParen => self.expr_group(),
            Identifier(ref name) => Ok(Expr::var(name)),
            _ => {
                Err(ParseError::token(
                    s_token,
                    // TODO: abstract token for literals
                    &[Number(0.0), String("".into()), False, True, Nil, LeftParen],
                ))
            }
        }
    }

    /// group → "(" expression ")" ;
    ///
    /// To be called after consuming "(" (predictive parsing).
    fn expr_group(&mut self) -> Result<Expr> {
        let expr = self.parse_expr()?;
        self.try_advance_if_find(&[Token::RightParen])?;
        Ok(expr)
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
