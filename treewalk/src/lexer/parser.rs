//! Crates an AST from a token stream
//!
//! We just need to peek one `Token` at a time

use crate::ast::stmt::{FnDeclArgs, Params};
use crate::ast::{expr::*, stmt::*};
use crate::lexer::token::*;
use std::iter::Peekable;
use std::rc::Rc;

// --------------------------------------------------------------------------------
// Errors

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken(UnexpectedTokenErrorArgs),
    NotAssignable(Expr),
}

impl ParseError {
    pub fn unexpected(found: &Token, expected: &[TokenKind]) -> Self {
        ParseError::UnexpectedToken(UnexpectedTokenErrorArgs::from_token(found, expected))
    }

    pub fn eof() -> Self {
        ParseError::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct UnexpectedTokenErrorArgs {
    pos: Location,
    expected: Vec<TokenKind>,
    found: TokenKind,
}

impl UnexpectedTokenErrorArgs {
    // TODO: more generic interface
    pub fn from_token(tk: &Token, expected: &[TokenKind]) -> Self {
        UnexpectedTokenErrorArgs {
            pos: tk.pos,
            expected: expected.iter().cloned().collect(),
            found: tk.kind.clone(),
        }
    }
}

// --------------------------------------------------------------------------------
// Parser

pub struct Parser<'a, I>
where
    I: Iterator<Item = &'a Token> + Sized,
{
    tks: Peekable<I>,
    counter: VarUseIdCounter,
}

impl<'a> Parser<'a, std::slice::Iter<'a, Token>> {
    // TODO: more abstarct constructor
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tks: tokens.iter().peekable(),
            counter: VarUseIdCounter::new(),
        }
    }
}

/// Iterator methods around `Peekable<I>`
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a Token> + Sized,
{
    fn peek(&mut self) -> Option<&&Token> {
        self.tks.peek()
    }

    fn next(&mut self) -> Option<&Token> {
        self.tks.next()
    }

    fn advance(&mut self) -> bool {
        self.tks.next().is_some()
    }

    /// Peek or else error
    fn try_peek(&mut self) -> Result<&&Token> {
        self.peek().ok_or(ParseError::eof())
    }

    /// Next or else error
    fn try_next(&mut self) -> Result<&Token> {
        self.next().ok_or(ParseError::eof())
    }

    /// Just a wrapper around `Iterator::find`.
    fn _find(tk: &Token, expected: &[TokenKind]) -> Option<TokenKind> {
        expected.iter().find(|t| t == &&tk.kind).map(|t| t.clone())
    }

    /// Safely tries to advance the token iterator
    fn consume(&mut self, expected: &TokenKind) -> Option<&Token> {
        match self.peek() {
            Some(tk) if tk.kind == *expected => Some(self.next().unwrap()),
            _ => None,
        }
    }

    /// Tries to consume the expected token or cause an error
    fn try_consume(&mut self, expected: &TokenKind) -> Result<&Token> {
        match self.peek() {
            Some(tk) if tk.kind == *expected => Ok(self.next().unwrap()),
            Some(tk) => Err(ParseError::unexpected(tk, &[expected.clone()])),
            None => Err(ParseError::eof()),
        }
    }

    fn try_consume_identifier(&mut self) -> Result<String> {
        if let Some(tk) = self.peek() {
            if let TokenKind::Ident(ref name) = tk.kind {
                let name = name.clone();
                self.advance();
                Ok(name)
            } else {
                Err(ParseError::unexpected(tk, &[TokenKind::Ident("".into())]))
            }
        } else {
            Err(ParseError::eof())
        }
    }

    // cannot identify token with fields
    fn consume_one_of(&mut self, expected: &[TokenKind]) -> Option<TokenKind> {
        let opt = Self::_find(self.peek()?, expected);
        if opt.is_some() {
            self.next();
        }
        opt
    }

    /// Just a wrapper of `Iterator::any`. Fails if nothing is found.
    ///
    /// Copies a `TokenKind` if it's found.
    fn _try_find(tk: &Token, expected: &[TokenKind]) -> Result<TokenKind> {
        Self::_find(tk, expected).ok_or(ParseError::unexpected(tk, expected))
    }
}

/// Statement / declaration parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a Token> + Sized,
{
    /// program → decl* EOF ;
    ///
    /// The entry point of the predictive parsing.
    pub fn parse(&mut self) -> (Vec<Stmt>, Vec<ParseError>) {
        let mut stmts = Vec::<Stmt>::new();
        let mut errors = Vec::<ParseError>::new();

        while let Some(tk) = self.decl() {
            match tk {
                Ok(stmt) => stmts.push(stmt),
                Err(why) => {
                    errors.push(why);
                    self.synchronize();
                }
            }
        }

        return (stmts, errors);
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            match self.try_peek()?.kind {
                TokenKind::RightBrace => {
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
        Ok(stmts)
    }

    /// Enters "panic mode" and tries to go to next statement.
    ///
    /// It goes to a next semicolon.
    fn synchronize(&mut self) {
        while let Some(tk) = self.peek() {
            let result = SyncPeekChecker::check_token(&tk.kind);
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
        Some(match self.peek()?.kind {
            TokenKind::Class => {
                self.advance();
                self.decl_class().map(|c| Stmt::Class(c))
            }
            TokenKind::Fn => {
                self.advance();
                self.decl_fn().map(|f| Stmt::Fn(f))
            }
            TokenKind::Var => {
                self.advance();
                self.decl_var()
            }
            _ => self.stmt(),
        })
    }

    /// declClass  → "class" IDENTIFIER "{" function* "}" ;
    fn decl_class(&mut self) -> Result<ClassDeclArgs> {
        let name = self.try_consume_identifier()?;
        self.try_consume(&TokenKind::LeftBrace)?;
        let mut methods = Vec::new();
        while self.consume(&TokenKind::Fn).is_some() {
            let method = self.decl_fn()?;
            methods.push(method);
        }
        self.try_consume(&TokenKind::RightBrace)?;
        Ok(ClassDeclArgs::new(name, methods))
    }

    /// declFn  → "fn" IDENTIFIER "(" params? ")" block ;
    fn decl_fn(&mut self) -> Result<FnDeclArgs> {
        let name = self.try_consume_identifier()?;

        self.try_consume(&TokenKind::LeftParen)?;
        let params = match self.try_peek()?.kind {
            TokenKind::RightParen => Vec::new(),
            _ => self.params()?,
        };
        self.try_consume(&TokenKind::RightParen)?;

        // we must first consume `{` to parse a block
        self.try_consume(&TokenKind::LeftBrace)?;
        let body = self.parse_block()?;

        Ok(FnDeclArgs::new(name, Rc::new(body), params))
    }

    /// params → IDENTIFIER ( "," IDENTIFIER )* ;
    fn params(&mut self) -> Result<Params> {
        let mut params = Vec::new();
        params.push(self.try_consume_identifier()?);
        while match self.peek() {
            Some(tk) if tk.kind == TokenKind::Comma => true,
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
        self.try_consume(&TokenKind::Eq)?;
        let init = self.expr()?;
        self.try_consume(&TokenKind::Semicolon)?;
        Ok(Stmt::var_dec(name, init))
    }

    /// stmt → exprStmt | printStmt | returnStmt whileStmt | block ;
    ///
    /// The root of predictive statement parsing. Sub rules are named as `stmt_xxx`.
    /// Note that sub rules don't consume unexpected tokens.
    pub fn stmt(&mut self) -> Result<Stmt> {
        use TokenKind::*;
        match &self.try_peek()?.kind {
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
        self.try_consume(&TokenKind::Semicolon)?;
        // TODO: adding Expr -> String functions for printing
        Ok(Stmt::print(expr))
    }

    /// block → "{" declaration* "}" ;
    ///
    /// Left brace `{` must be consumed before calling this.
    pub fn stmt_block(&mut self) -> Result<BlockArgs> {
        Ok(BlockArgs {
            stmts: self.parse_block()?,
        })
    }

    /// if → "if" expr block elseRecursive
    pub fn stmt_if(&mut self) -> Result<Stmt> {
        // TODO: no overhead
        let if_ = self.parse_if()?;
        Ok(Stmt::If(Box::new(if_)))
    }

    fn parse_if(&mut self) -> Result<IfArgs> {
        let condition = self.expr()?;
        self.try_consume(&TokenKind::LeftBrace)?;
        let if_true = self.stmt_block()?;
        let if_false = self._else_recursive()?;
        Ok(IfArgs::new(condition, if_true, if_false))
    }

    /// elseRecursive → ("else" "if" block)* ("else" block)?
    fn _else_recursive(&mut self) -> Result<Option<ElseBranch>> {
        if self.consume(&TokenKind::Else).is_none() {
            return Ok(None);
        }
        let tk = self.try_peek()?;
        match tk.kind {
            // else if
            TokenKind::If => {
                self.advance();
                let else_if = self.parse_if()?;
                Ok(Some(ElseBranch::else_if(else_if)))
            }
            // else
            TokenKind::LeftBrace => {
                self.advance();
                let else_ = self.parse_block()?;
                // TODO: no overhead with new type pattern
                Ok(Some(ElseBranch::JustElse(BlockArgs { stmts: else_ })))
            }
            // error
            _ => Err(ParseError::unexpected(
                tk,
                &[TokenKind::If, TokenKind::LeftBrace],
            )),
        }
    }

    /// stmtReturn → "return" expression? ";" ;
    pub fn stmt_return(&mut self) -> Result<Stmt> {
        let expr = self.expr()?;
        self.try_consume(&TokenKind::Semicolon)?;
        Ok(Stmt::return_(expr))
    }

    /// while → "while" expr block
    pub fn stmt_while(&mut self) -> Result<Stmt> {
        let condition = self.expr()?;
        self.try_consume(&TokenKind::LeftBrace)?;
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
        self.try_consume(&TokenKind::Semicolon)?;
        Ok(Stmt::expr(expr))
    }
}

/// Expression parsing
impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = &'a Token> + Sized,
{
    /// rrp → Subruple (Oper Subrule)*
    ///
    /// Abstracts right recursive parsing.
    #[inline]
    fn rrp<Oper, SubRule, Folder>(
        &mut self,
        sub_rule: SubRule,
        delimiters: &[TokenKind],
        folder: Folder,
    ) -> Result<Expr>
    where
        TokenKind: Into<Option<Oper>>,
        SubRule: Fn(&mut Self) -> Result<Expr>,
        Folder: Fn(Expr, Oper, Expr) -> Expr,
    {
        let mut expr = sub_rule(self)?;
        while let Some(token) = self.consume_one_of(delimiters) {
            let right = sub_rule(self)?;
            let oper = token.into().unwrap();
            expr = folder(expr, oper, right);
        }
        Ok(expr)
    }

    /// expr → assignment
    pub fn expr(&mut self) -> Result<Expr> {
        self.expr_assign()
    }

    /// assignment → ( call "." )? IDENTIFIER "=" assignment
    ///            | logic_or;
    fn expr_assign(&mut self) -> Result<Expr> {
        let lhs = self.expr_or()?;

        // peek to see if it's an assignment
        if self.consume(&TokenKind::Eq).is_none() {
            return Ok(lhs);
        };

        match lhs {
            // assign
            Expr::Variable(ref var) => {
                let rhs = self.expr_assign()?;
                return Ok(Expr::assign(&var.name, rhs, self.counter.next()));
            }
            // set (assign to get expression)
            Expr::Get(get) => {
                // e.g. x.y.z = 3;  // x, y are Expr::Get, z is Expr::Set
                let name = get.name.clone();
                let rhs = self.expr_assign()?;
                return Ok(Expr::set(get.body, &name, rhs));
            }
            // error
            _ => {
                return Err(ParseError::NotAssignable(lhs));
            }
        };
    }

    /// logic_or → logicAnd ("||" logicAnd)*
    fn expr_or(&mut self) -> Result<Expr> {
        self.rrp(&Self::expr_and, &[TokenKind::Or], &Expr::logic)
    }

    /// logic_and → eq (&& eq)*
    fn expr_and(&mut self) -> Result<Expr> {
        self.rrp(&Self::expr_eq, &[TokenKind::And], &Expr::logic)
    }

    /// eq → cmp ( ( "!=" | "==" ) cmp )* ;
    fn expr_eq(&mut self) -> Result<Expr> {
        use TokenKind::*;
        self.rrp(&Self::expr_cmp, &[EqEq, BangEq], &Expr::binary)
    }

    /// cmp → add ( ( ">" | ">=" | "<" | "<=" ) add )* ;
    fn expr_cmp(&mut self) -> Result<Expr> {
        use TokenKind::*;
        self.rrp(
            &Self::expr_add,
            &[Greater, GreaterEq, Less, LessEq],
            &Expr::binary,
        )
    }

    /// add → mul ( ( "-" | "+" ) mul )* ;
    fn expr_add(&mut self) -> Result<Expr> {
        use TokenKind::*;
        self.rrp(&Self::expr_mul, &[Plus, Minus], &Expr::binary)
    }

    /// mul → unary ( ( "/" | "*" ) unary )* ;
    fn expr_mul(&mut self) -> Result<Expr> {
        use TokenKind::*;
        self.rrp(&Self::expr_unary, &[Slash, Star], &Expr::binary)
    }

    /// unary → ( "!" | "-" ) unary | call ;
    fn expr_unary(&mut self) -> Result<Expr> {
        use TokenKind::*;
        match self.try_peek()?.kind {
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

    /// call → primary (invoke|prop)* ;
    fn expr_call(&mut self) -> Result<Expr> {
        let mut expr = self.expr_prim()?;

        // TODO: use right recursive parsing
        loop {
            match self.try_peek()?.kind {
                TokenKind::LeftParen => {
                    // invoke → "(" args ")"
                    self.advance();
                    let args = if self.try_peek()?.kind == TokenKind::RightParen {
                        self.advance();
                        Vec::new()
                    } else {
                        let args = self.expr_call_args()?;
                        args
                    };
                    expr = Expr::call(expr, args);
                }

                TokenKind::Dot => {
                    self.advance();
                    let name = self.try_consume_identifier()?;
                    expr = Expr::get(expr, &name);
                }

                _ => {
                    return Ok(expr);
                }
            }
        }
    }

    /// args → expr ( "," expr )* ;
    // TODO: use rrp
    fn expr_call_args(&mut self) -> Result<Args> {
        let mut args = Args::new();
        args.push(self.expr()?);
        loop {
            match self.try_peek()? {
                tk if tk.kind == TokenKind::Comma => {
                    self.advance();
                    args.push(self.expr()?);
                }
                tk if tk.kind == TokenKind::RightParen => {
                    self.advance();
                    return Ok(args);
                }
                tk => {
                    return Err(ParseError::unexpected(
                        tk,
                        &[TokenKind::Comma, TokenKind::RightParen],
                    ));
                }
            }
        }
    }

    /// primary → literal | group | indentifier | self ;
    ///
    /// literal → number | string | "false" | "true" | "nil" ;
    /// group   → "(" expression ")" ;
    ///
    /// Make sure that there exists next token (predictive parsing).
    fn expr_prim(&mut self) -> Result<Expr> {
        // TODO: refactor
        let mut var = {
            let tk = self.try_next()?;
            use TokenKind::*;
            let name = match tk.kind {
                Ident(ref name) => name,
                LeftParen => return self.expr_group(),
                Self_ => return Ok(Expr::Self_(SelfData {})),
                _ => {
                    if let Some(literal) = LiteralData::from_token(&tk.kind) {
                        return Ok(literal.into());
                    }
                    return Err(ParseError::unexpected(
                        tk,
                        // TODO: abstract token for literals
                        &[Num(0.0), Str("".into()), False, True, Nil, LeftParen],
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
        self.try_consume(&TokenKind::RightParen)?;
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
    pub fn check_token<T: Borrow<TokenKind>>(token: T) -> Self {
        use TokenKind::*;
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
