use crate::abs::expr::*;
use crate::abs::stmt::*;
use crate::abs::token::*;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken(UnexpectedTokenArgs),
}

impl ParseError {
    pub fn unexpected(found: &SourceToken, expected: &[Token]) -> Self {
        ParseError::UnexpectedToken(UnexpectedTokenArgs::from_s_token(found, expected))
    }
    pub fn not_beginning_of_stmt(found: &SourceToken) -> Self {
        use Token::*;
        Self::unexpected(found, &[Print])
    }
    pub fn eof() -> Self {
        ParseError::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct UnexpectedTokenArgs {
    pos: SourcePosition,
    expected: Vec<Token>,
    found: Token,
}

impl UnexpectedTokenArgs {
    // TODO: more generic interface
    pub fn from_s_token(s_token: &SourceToken, expected: &[Token]) -> Self {
        UnexpectedTokenArgs {
            pos: s_token.pos,
            expected: expected.iter().cloned().collect(),
            found: s_token.token.clone(),
        }
    }
}

type Result<T> = std::result::Result<T, ParseError>;

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

// Impl block of helper functions
// TODO: which to return: SourceToken or Token
// TODO: Token vs &Token
// TODO: lifetime? (read the rust book first)
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    fn peek(&mut self) -> Option<&&SourceToken> {
        self.tokens.peek()
    }

    fn advance(&mut self) -> Option<&SourceToken> {
        self.tokens.next()
    }

    // TODO: benchmark performance between self.try_peek()?; and match self.peek() { .. }
    fn try_peek(&mut self) -> Result<&&SourceToken> {
        self.peek().ok_or(ParseError::eof())
    }

    fn try_advance(&mut self) -> Result<&SourceToken> {
        self.advance().ok_or(ParseError::eof())
    }

    fn _find(s_token: &SourceToken, expected: &[Token]) -> Option<Token> {
        expected.iter().cloned().find(|t| t == &s_token.token)
    }

    fn _any(s_token: &SourceToken, expected: &[Token]) -> bool {
        expected.iter().any(|t| t == &s_token.token)
    }

    fn peek_match(&mut self, expected: &[Token]) -> Option<&&SourceToken> {
        let s_token = self.peek()?;
        if Self::_any(s_token, expected) {
            Some(s_token)
        } else {
            None
        }
    }

    fn advance_if_match(&mut self, expected: &[Token]) -> Option<bool> {
        if Self::_any(self.peek()?, expected) {
            self.advance();
            Some(true)
        } else {
            Some(false)
        }
    }

    fn try_match(s_token: &SourceToken, expected: &[Token]) -> Result<Token> {
        Self::_find(s_token, expected).ok_or(ParseError::unexpected(s_token, expected))
    }

    fn try_peek_match(&mut self, expected: &[Token]) -> Result<Token> {
        self.try_peek()
            .and_then(|s_token| Self::try_match(s_token, expected))
    }

    /// True if any matches to the next token.
    fn try_advance_match(&mut self, expected: &[Token]) -> Result<Token> {
        self.try_advance()
            .and_then(|s_token| Self::try_match(s_token, expected))
    }

    fn try_advance_if_match(&mut self, expected: &[Token]) -> Result<Token> {
        let s_token = self.try_peek()?;
        if let Some(token) = Self::_find(s_token, expected) {
            self.advance();
            Ok(token)
        } else {
            Err(ParseError::unexpected(s_token, expected))
        }
    }
}

// Impl block of parse functions
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    /// program     → declaration* EOF ;
    /// Each rule is to be entered when it is expected or after some token for the beginning of it is consumed.
    pub fn parse(&mut self) -> (Vec<Stmt>, Vec<ParseError>) {
        let mut stmts = Vec::<Stmt>::new();
        let mut errors = Vec::<ParseError>::new();

        // TODO: making sure the next token is not None
        // TODo: so that following functions dont need to care about it.
        while let Some(s_token) = self.parse_declaration() {
            match s_token {
                Ok(stmt) => stmts.push(stmt),
                Err(why) => {
                    errors.push(why);
                    // not so good though
                    self.synchronize();
                }
            }
        }

        return (stmts, errors);
    }

    /// declaration → varDecl | statement ;
    fn parse_declaration(&mut self) -> Option<Result<Stmt>> {
        Some(if self.advance_if_match(&[Token::Var])? {
            self.stmt_var()
        } else {
            self.parse_stmt()
        })
    }

    /// varDecl → "var" IDENTIFIER "=" expression ";" ;
    ///
    /// Different from the book, "=" is always required.
    fn stmt_var(&mut self) -> Result<Stmt> {
        let s_token = self.try_peek()?;
        if let Token::Identifier(ref name) = s_token.token {
            let name = name.clone();
            self.try_advance_if_match(&[Token::Equal])?;
            let expr = self.parse_expr()?;
            self.try_advance_if_match(&[Token::Semicolon])?;
            Ok(Stmt::var_dec(name, expr))
        } else {
            Err(ParseError::unexpected(
                s_token,
                &[Token::Identifier("".into())],
            ))
        }
    }

    /// Returns some result until finding EoF.
    /// Sub rules don't consume an unexpected token.
    /// statement   → exprStmt | printStmt ;
    pub fn parse_stmt(&mut self) -> Result<Stmt> {
        use Token::*;
        match &self.try_advance()?.token {
            Print => self.stmt_print(),
            _ => self.stmt_expr(),
        }
    }

    fn stmt_print(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;
        self.try_advance_if_match(&[Token::Semicolon])?;
        // TODO: evaluate expression
        Ok(Stmt::print(format!("{:?}", expr)))
    }

    /// Sub rules don't consume an unexpected token.
    fn stmt_expr(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;
        Ok(Stmt::expr(expr))
    }
}

// Impl block of expression parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.expr_equality()
    }

    /// Right recursive parsing: Expr (Oper Expr)*
    /// Expr is created by sub_rule, which may return error.
    /// Doesn't consume unexpected token.
    // TODO: #[inline], macro, etc for performance.
    fn rrp<Oper>(
        &mut self,
        sub_rule: &Fn(&mut Self) -> Result<Expr>,
        delimiters: &[Token],
        folder: &Fn(Expr, Oper, Expr) -> Expr,
    ) -> Result<Expr>
    where
        Token: Into<Option<Oper>>,
    {
        let mut expr = sub_rule(self)?;
        // TODO: advance_if_match and panic mode
        // TODO: Oper: From<&Token>
        while let Some(token) = self.peek_match(delimiters).map(|s| s.token.clone()) {
            self.advance();
            let right = sub_rule(self)?;
            let oper = token.into().unwrap();
            expr = folder(expr, oper, right);
        }
        return Ok(expr);
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn expr_equality(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp::<BinaryOper>(
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

    /// unary   → ( "!" | "-" ) unary | primary ;
    fn expr_unary(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_primary, &[Bang, Minus], &Expr::binary)
    }

    /// primary → LITERAL | GROUP | IDENTIFIER ;
    ///
    /// literal → NUMBER | STRING | "false" | "true" | "nil" ;
    /// group → "(" expr ")"
    /// identifier →
    fn expr_primary(&mut self) -> Result<Expr> {
        // TODO: use match only once: the following line means opt.ok_or()?;
        let s_token = self.try_advance()?;
        if let Some(args) = LiteralArgs::from_token(&s_token.token) {
            Ok(args.into())
        // } else let Identifier(ref name) = s_token.token {
        } else if s_token.token == Token::LeftParen {
            return self.expr_group();
        } else {
            use Token::*;
            Err(ParseError::unexpected(
                s_token,
                &[Number(0.0), String("".into()), False, True, Nil, LeftParen],
            ))
        }
    }

    /// group → "(" expression ")"
    /// To be called after consuming "("
    fn expr_group(&mut self) -> Result<Expr> {
        // TODO: synchronize
        let expr = self.parse_expr()?;
        self.try_advance_if_match(&[Token::RightParen])?;
        Ok(expr)
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
