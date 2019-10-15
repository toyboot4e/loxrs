use crate::ast::stmt::{FnDeclArgs, Params};
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
    pub fn unexpected(found: &SourceToken, expected: &[Token]) -> Self {
        ParseError::UnexpectedToken(UnexpectedTokenErrorArgs::from_s_token(found, expected))
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
    counter: VarUseIdCounter,
}

impl<'a> Parser<'a, std::slice::Iter<'a, SourceToken>> {
    // TODO: more abstarct constructor
    pub fn new(tokens: &'a [SourceToken]) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
            counter: VarUseIdCounter::new(),
        }
    }
}

/// Iterator methods around `Peekable<I>`
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

    fn try_next(&mut self) -> Result<&SourceToken> {
        self.next().ok_or(ParseError::eof())
    }

    /// Just a wrapper around `Iterator::find`.
    fn _find(s_token: &SourceToken, expected: &[Token]) -> Option<Token> {
        expected
            .iter()
            .find(|t| t == &&s_token.token)
            .map(|t| t.clone())
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

    /// Safely tries to advance the token iterator
    fn consume(&mut self, expected: &Token) -> Option<&SourceToken> {
        match self.peek() {
            Some(s_token) if s_token.token == *expected => Some(self.next().unwrap()),
            _ => None,
        }
    }

    /// Tries to consume the expected token or cause an error
    fn try_consume(&mut self, expected: &Token) -> Result<&SourceToken> {
        match self.peek() {
            Some(s_token) if s_token.token == *expected => Ok(self.next().unwrap()),
            Some(s_token) => Err(ParseError::unexpected(s_token, &[expected.clone()])),
            None => Err(ParseError::eof()),
        }
    }

    fn try_consume_identifier(&mut self) -> Result<String> {
        if let Some(s_token) = self.peek() {
            if let Token::Identifier(ref name) = s_token.token {
                let name = name.clone();
                self.advance();
                Ok(name)
            } else {
                Err(ParseError::unexpected(
                    s_token,
                    &[Token::Identifier("".into())],
                ))
            }
        } else {
            Err(ParseError::eof())
        }
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
        Self::_find(s_token, expected).ok_or(ParseError::unexpected(s_token, expected))
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

/// Statement / declaration parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a SourceToken> + Sized,
{
    /// program → decl* EOF ;
    ///
    /// The entry point of the predictive parsing.
    pub fn parse(&mut self) -> (Vec<Stmt>, Vec<ParseError>) {
        let mut stmts = Vec::<Stmt>::new();
        let mut errors = Vec::<ParseError>::new();

        while let Some(s_token) = self.decl() {
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

    /// decl → declClass | declFn | declVar | stmt ;
    ///
    /// The root of parsing.
    fn decl(&mut self) -> Option<Result<Stmt>> {
        Some(match self.peek()?.token {
            Token::Class => {
                self.advance();
                self.decl_class().map(|c| Stmt::Class(c))
            }
            Token::Fn => {
                self.advance();
                self.decl_fn().map(|f| Stmt::Fn(f))
            }
            Token::Var => {
                self.advance();
                self.decl_var()
            }
            _ => self.stmt(),
        })
    }

    /// declClass  → "class" IDENTIFIER "{" function* "}" ;
    fn decl_class(&mut self) -> Result<ClassDeclArgs> {
        let name = self.try_consume_identifier()?;
        self.try_consume(&Token::LeftBrace)?;
        let mut methods = Vec::new();
        while self.consume(&Token::Fn).is_some() {
            let method = self.decl_fn()?;
            methods.push(method);
        }
        self.try_consume(&Token::RightBrace)?;
        Ok(ClassDeclArgs::new(name, methods))
    }

    /// declFn  → "fn" IDENTIFIER "(" params? ")" block ;
    fn decl_fn(&mut self) -> Result<FnDeclArgs> {
        let name = self.try_consume_identifier()?;

        self.try_consume(&Token::LeftParen)?;
        let params = match self.try_peek()?.token {
            // TODO: reduce reproducing the token
            Token::Identifier(_) => Some(self.params()?),
            _ => None,
        };
        self.try_consume(&Token::RightParen)?;

        // we must first consume `{` to parse a block
        self.try_consume(&Token::LeftBrace)?;
        let body = self.stmt_block()?;

        Ok(FnDeclArgs::new(name, body, params))
    }

    /// params → IDENTIFIER ( "," IDENTIFIER )* ;
    fn params(&mut self) -> Result<Params> {
        let mut params = Vec::new();
        params.push(self.try_consume_identifier()?);
        while match self.peek() {
            Some(s_token) if s_token.token == Token::Comma => true,
            _ => false,
        } {
            self.advance();
            params.push(self.try_consume_identifier()?);
        }
        Ok(params)
    }

    /// declVar → "var" IDENTIFIER "=" expression ";" ;
    ///
    /// It always requires initializer, different from the original Lox.
    /// Call it after consuming `var`.
    fn decl_var(&mut self) -> Result<Stmt> {
        let name = self.try_consume_identifier()?;
        self.try_consume(&Token::Equal)?;
        let init = self.expr()?;
        self.try_consume(&Token::Semicolon)?;
        Ok(Stmt::var_dec(name, init))
    }

    /// stmt → exprStmt | printStmt | returnStmt whileStmt | block ;
    ///
    /// The root of predictive statement parsing. Sub rules are named as `stmt_xxx`.
    /// Note that sub rules don't consume unexpected tokens.
    pub fn stmt(&mut self) -> Result<Stmt> {
        use Token::*;
        match &self.try_peek()?.token {
            Print => {
                self.next();
                self.stmt_print()
            }
            LeftBrace => {
                self.next();
                Ok(self.stmt_block()?.into_stmt())
            }
            If => {
                self.next();
                self.stmt_if()
            }
            Return => {
                self.next();
                self.stmt_return()
            }
            While => {
                self.next();
                self.stmt_while()
            }
            _ => self.stmt_expr(),
        }
    }

    /// printStmt → "print" expression ";" ;
    ///
    /// To be called after consuming `print` (predictive parsing).
    fn stmt_print(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.try_consume(&Token::Semicolon)?;
        // TODO: adding Expr -> String functions for printing
        Ok(Stmt::print(expr))
    }

    /// block → "{" declaration* "}" ;
    ///
    /// Left brace `{` must be consumed before calling this.
    pub fn stmt_block(&mut self) -> Result<BlockArgs> {
        let mut stmts = Vec::new();
        loop {
            match self.try_peek()?.token {
                Token::RightBrace => {
                    self.advance();
                    break;
                }
                _ => {
                    let stmt = self
                        .decl()
                        .unwrap_or_else(|| Err(ParseError::UnexpectedEof))?;
                    stmts.push(stmt);
                }
            };
        }
        Ok(BlockArgs { stmts: stmts })
    }

    /// if → "if" expr block elseRecursive
    pub fn stmt_if(&mut self) -> Result<Stmt> {
        let condition = self.expr()?;
        self.try_consume(&Token::LeftBrace)?;
        let if_true = self.stmt_block()?.into_stmt();
        let if_false = self._else_recursive()?;
        Ok(Stmt::if_then_else(condition, if_true, if_false))
    }

    /// elseRecursive → ("else" "if" block)* ("else" block)?
    fn _else_recursive(&mut self) -> Result<Option<Stmt>> {
        if self.consume(&Token::Else).is_none() {
            return Ok(None);
        }
        let s_token = self.try_peek()?;
        match s_token.token {
            // else if
            Token::If => {
                self.advance();
                let else_if = self.stmt_if()?;
                Ok(Some(else_if))
            }
            // else
            Token::LeftBrace => {
                self.advance();
                let else_ = self.stmt_block()?.into_stmt();
                Ok(Some(else_))
            }
            // error
            _ => Err(ParseError::unexpected(
                s_token,
                &[Token::If, Token::LeftBrace],
            )),
        }
    }

    /// stmtReturn → "return" expression? ";" ;
    pub fn stmt_return(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.try_consume(&Token::Semicolon)?;
        Ok(Stmt::return_(expr))
    }

    /// while → "while" expr block
    pub fn stmt_while(&mut self) -> Result<Stmt> {
        let condition = self.expr()?;
        self.try_consume(&Token::LeftBrace)?;
        let block = self.stmt_block()?;
        Ok(Stmt::while_(condition, block))
    }

    /// Expression statement or (recursive) assignment
    ///
    /// exprStmt → IDENTIFIER "=" assignment
    ///          | equality ;
    // FIXME: no need to enter sync mode but returns Error
    fn stmt_expr(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.try_consume(&Token::Semicolon)?;
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
        Ok(expr)
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
    pub fn expr(&mut self) -> Result<Expr> {
        self.assignment()
    }

    /// assignment → IDENTIFIER "=" assignment
    ///            | logic_or ;
    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.expr_or()?; // may be an identifier

        // peek to see if it's an assignment
        match self.try_peek()?.token {
            Token::Equal => {}
            _ => {
                return Ok(expr);
            }
        };

        // previous `Expr` must be assignable (`Expr::Variable`)
        let name = match expr {
            Expr::Variable(ref var) => &var.name,
            e => return Err(ParseError::NotAssignable(e)),
        };
        self.advance(); // =
        let right = self.assignment()?;
        Ok(Expr::assign(name, right, self.counter.next()))
    }

    /// logic_or → logicAnd ("||" logicAnd)*
    fn expr_or(&mut self) -> Result<Expr> {
        self.rrp(&Self::expr_and, &[Token::Or], &Expr::logic)
    }

    /// logic_and → eq (&& eq)*
    fn expr_and(&mut self) -> Result<Expr> {
        self.rrp(&Self::expr_eq, &[Token::And], &Expr::logic)
    }

    /// eq → cmp ( ( "!=" | "==" ) cmp )* ;
    fn expr_eq(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_cmp, &[EqualEqual, BangEqual], &Expr::binary)
    }

    /// cmp → add ( ( ">" | ">=" | "<" | "<=" ) add )* ;
    fn expr_cmp(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(
            &Self::expr_add,
            &[Greater, GreaterEqual, Less, LessEqual],
            &Expr::binary,
        )
    }

    /// add → mul ( ( "-" | "+" ) mul )* ;
    fn expr_add(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_mul, &[Plus, Minus], &Expr::binary)
    }

    /// mul → unary ( ( "/" | "*" ) unary )* ;
    fn expr_mul(&mut self) -> Result<Expr> {
        use Token::*;
        self.rrp(&Self::expr_unary, &[Slash, Star], &Expr::binary)
    }

    /// unary → ( "!" | "-" ) unary | primary | call ;
    fn expr_unary(&mut self) -> Result<Expr> {
        use Token::*;
        match self.try_peek()?.token {
            Bang => {
                self.advance();
                Ok(Expr::unary(UnaryOper::Not, self.expr_unary()?))
            }
            Minus => {
                self.advance();
                Ok(Expr::unary(UnaryOper::Minus, self.expr_unary()?))
            }
            _ => self.expr_call(),
        }
    }

    /// call → primary invoke* ;
    ///
    /// invoke → "(" args ")" ;
    /// args → expr ( "," expr )* ;
    fn expr_call(&mut self) -> Result<Expr> {
        let mut expr = self.expr_prim()?;
        if self.try_peek()?.token != Token::LeftParen {
            return Ok(expr); // prim
        }

        // TODO: use right recursive parsing
        // invoke*
        while self.consume(&Token::LeftParen).is_some() {
            let args = if self.try_peek()?.token == Token::RightParen {
                None
            } else {
                Some(self.expr_call_args()?)
            };
            self.try_consume(&Token::RightParen)?;
            expr = Expr::call(expr, args);
        }

        Ok(expr)
    }

    /// args → expr ( "," expr )* ;
    // TODO: use rrp
    fn expr_call_args(&mut self) -> Result<Args> {
        let mut args = Args::new();
        args.push(self.expr()?);
        loop {
            match self.try_peek()? {
                s_token if s_token.token == Token::Comma => {
                    args.push(self.expr()?);
                }
                s_token if s_token.token == Token::RightParen => {
                    return Ok(args);
                }
                s_token => {
                    return Err(ParseError::unexpected(
                        s_token,
                        &[Token::Comma, Token::RightParen],
                    ));
                }
            }
        }
    }

    /// primary → literal | group |indentifier ;
    ///
    /// literal → number | string | "false" | "true" | "nil" ;
    /// group   → "(" expression ")" ;
    ///
    /// Make sure that there exists next token (predictive parsing).
    fn expr_prim(&mut self) -> Result<Expr> {
        // TODO: refactor
        let mut var = {
            let s_token = self.try_next()?;
            if let Some(literal) = LiteralData::from_token(&s_token.token) {
                return Ok(literal.into());
            }
            use Token::*;
            let name = match s_token.token {
                LeftParen => return self.expr_group(),
                Identifier(ref name) => name,
                _ => {
                    return Err(ParseError::unexpected(
                        s_token,
                        // TODO: abstract token for literals
                        &[Number(0.0), String("".into()), False, True, Nil, LeftParen],
                    ));
                }
            };
            VarUseData::new(name, VarUseId::new())
        };
        var.id = self.counter.next();
        Ok(Expr::Variable(var))
    }

    /// group → "(" expression ")" ;
    ///
    /// To be called after consuming "(" (predictive parsing).
    fn expr_group(&mut self) -> Result<Expr> {
        let expr = self.expr()?;
        self.try_consume(&Token::RightParen)?;
        Ok(expr)
    }
}

/// This is for panic mode (synchronizing)
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
            Class | Fn | Var | If | For | While | Print | Return => Self {
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
