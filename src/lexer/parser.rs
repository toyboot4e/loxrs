use crate::ast::{expr::*, stmt::*};
use crate::lexer::token::*;
use std::iter::Peekable;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    // TODO: EoF error
    UnexpectedEof,
    UnexpectedToken(UnexpectedTokenErrorArgs),
    NotAssignable(Expr),
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

    // FIXME: cannot identify token with fields
    fn consume_any_of(&mut self, expected: &[Token]) -> Option<Token> {
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
    fn try_consume_any_of(&mut self, expected: &[Token]) -> Result<Token> {
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

    /// root → varDecl | statement ;
    ///
    /// The root of predictive parsing. Sub rules are named as `stmt_xxx`.
    fn parse_any(&mut self) -> Option<Result<Stmt>> {
        Some(if self.advance_if_any(&[Token::Var])? {
            self.stmt_var_decl()
        } else {
            self.parse_stmt()
        })
    }

    /// varDecl → "var" IDENTIFIER "=" expression ";" ;
    ///
    /// The initializer is always required, different from the original Lox.
    fn stmt_var_decl(&mut self) -> Result<Stmt> {
        let s_token = self.try_peek()?;
        if let Token::Identifier(ref name) = s_token.token {
            let name = name.clone(); // releases &mut self
            self.next(); // consuming the identifier
            self.try_consume_any_of(&[Token::Equal])?;
            let init = self.parse_expr()?;
            self.try_consume_any_of(&[Token::Semicolon])?;
            Ok(Stmt::var_dec(name, init))
        } else {
            Err(ParseError::token(s_token, &[Token::Identifier("".into())]))
        }
    }

    /// statement → exprStmt | printStmt | block ;
    ///
    /// Note that sub rules don't consume unexpected tokens.
    pub fn parse_stmt(&mut self) -> Result<Stmt> {
        use Token::*;
        match &self.try_peek()?.token {
            Print => {
                self.next();
                self.stmt_print()
            }
            LeftBrace => {
                self.next();
                self.stmt_block()
            }
            If => {
                self.next();
                self.stmt_if()
            }
            _ => self.stmt_expr(),
        }
    }

    /// block → "{" declaration* "}" ;
    pub fn stmt_block(&mut self) -> Result<Stmt> {
        let mut stmts = Vec::new();
        loop {
            match self.try_peek()?.token {
                Token::RightBrace => {
                    self.advance();
                    return Ok(Stmt::Block(stmts));
                }
                _ => {
                    let stmt = self
                        .parse_any()
                        .unwrap_or_else(|| Err(ParseError::UnexpectedEof))?;
                    stmts.push(stmt);
                }
            };
        }
    }

    /// printStmt → "print" expression ";" ;
    ///
    /// To be called after consuming `print` (predictive parsing).
    fn stmt_print(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;
        self.try_consume_any_of(&[Token::Semicolon])?;
        // TODO: adding Expr -> String functions for printing
        Ok(Stmt::print(expr))
    }

    /// if → "if" expr block elseRecursive
    pub fn stmt_if(&mut self) -> Result<Stmt> {
        let condition = self.parse_expr()?;
        self.try_consume_any_of(&[Token::LeftBrace])?;
        let if_true = self.stmt_block()?;
        let if_false = self._else_recursive()?;
        Ok(Stmt::if_then_else(condition, if_true, if_false))
    }

    /// elseRecursive → ("else" "if" block)* ("else" block)?
    fn _else_recursive(&mut self) -> Result<Option<Stmt>> {
        match self.peek() {
            Some(s_token) if s_token.token == Token::Else => {
                self.advance();
                match self.try_peek()? {
                    // else if
                    s_token if s_token.token == Token::If => {
                        self.advance();
                        let else_if = self.stmt_if()?;
                        Ok(Some(else_if))
                    }
                    // else
                    s_token if s_token.token == Token::LeftBrace => {
                        self.advance();
                        let else_ = self.stmt_block()?;
                        Ok(Some(else_))
                    }
                    // else <unexpected>
                    s_token => Err(ParseError::token(s_token, &[Token::If, Token::LeftBrace])),
                }
            }
            // EoF or not if-else related token
            _ => Ok(None),
        }
    }

    /// Expression statement or (recursive) assignment
    ///
    /// exprStmt → IDENTIFIER "=" assignment
    ///          | equality ;
    // FIXME: no need to enter sync mode but returns Error
    fn stmt_expr(&mut self) -> Result<Stmt> {
        let expr = self.parse_expr()?;
        self.try_consume_any_of(&[Token::Semicolon])?;
        Ok(Stmt::expr(expr))
    }
}

/// Expression parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    /// rrp → Subruple (Oper Subrule)*
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
        while let Some(token) = self.consume_any_of(delimiters) {
            let right = sub_rule(self)?;
            let oper = token.into().unwrap();
            expr = folder(expr, oper, right);
        }
        return Ok(expr);
    }

    /// `Folder` can fail. Left and right rules may be different
    #[inline]
    fn rrp_2<Oper, SubRule, Folder>(
        &mut self,
        left: Expr,
        sub_rule: SubRule,
        delimiters: &[Token],
        folder: Folder,
    ) -> Result<Expr>
    where
        Token: Into<Option<Oper>>,
        SubRule: Fn(&mut Self) -> Result<Expr>,
        Folder: Fn(Expr, Oper, Expr) -> Result<Expr>,
    {
        let mut expr = left;
        while let Some(token) = self.consume_any_of(delimiters) {
            let right = sub_rule(self)?;
            let oper = token.into().unwrap();
            expr = folder(expr, oper, right)?;
        }
        return Ok(expr);
    }

    /// expr → assignment
    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.assignment()
    }

    /// exprStmt → IDENTIFIER "=" assignment
    ///          | logic_or ;
    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.expr_or()?; // may be an identifier

        // may be assignment (or semicolon must exist)
        if self.try_peek()?.token == Token::Equal {
            // previous `Expr` must be assignable (`Expr::Variable`)
            let name = match expr {
                Expr::Variable(ref name) => name,
                e => return Err(ParseError::NotAssignable(e)),
            };
            // right recursive parsing
            self.advance(); // =
            let right = self.assignment()?;
            Ok(Expr::Assign(Box::new(AssignArgs {
                name: name.clone(),
                expr: right,
            })))
        } else {
            Ok(expr)
        }
    }

    /// logic_or → logicAnd ("||" logicAnd)*
    fn expr_or(&mut self) -> Result<Expr> {
        self.rrp(&Self::expr_and, &[Token::Or], &Expr::logic)
    }

    /// logic_and → equality (&& equality)*
    fn expr_and(&mut self) -> Result<Expr> {
        self.rrp(&Self::expr_equality, &[Token::And], &Expr::logic)
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn expr_equality(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_cmp, &[EqualEqual, BangEqual], &Expr::binary)
    }

    /// comparison → addition ( ( ">" | ">=" | "<" | "<=" ) addition )* ;
    fn expr_cmp(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(
            &Self::expr_add,
            &[Greater, GreaterEqual, Less, LessEqual],
            &Expr::binary,
        )
    }

    /// addition → multiplication ( ( "-" | "+" ) multiplication )* ;
    fn expr_add(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_mul, &[Plus, Minus], &Expr::binary)
    }

    /// multiplication → unary ( ( "/" | "*" ) unary )* ;
    fn expr_mul(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_unary, &[Slash, Star], &Expr::binary)
    }

    /// unary → ( "!" | "-" ) unary | primary ;
    fn expr_unary(&mut self) -> Result<Expr> {
        use Token::*;
        match self.try_peek()?.token {
            Minus => {
                self.advance();
                Ok(Expr::unary(UnaryOper::Minus, self.expr_unary()?))
            }
            Bang => {
                self.advance();
                Ok(Expr::unary(UnaryOper::Not, self.expr_unary()?))
            }
            _ => self.expr_prim(),
        }
    }

    /// primary → literal | group | indentifier ;
    ///
    /// literal → number | string | "false" | "true" | "nil" ;
    /// group   → "(" expression ")" ;
    ///
    /// Make sure that there exists next token (predictive parsing).
    fn expr_prim(&mut self) -> Result<Expr> {
        let s_token = self.try_advance()?;
        if let Some(literal) = LiteralArgs::from_token(&s_token.token) {
            return Ok(literal.into());
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
        self.try_consume_any_of(&[Token::RightParen])?;
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
