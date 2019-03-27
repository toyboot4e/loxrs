use crate::token::Token;
use std::convert::From;
// TODO: benchmark lazy static vs match
// TODO: combining oper and token or not

// specific subtypes are always need for the pretty printer.
pub enum Expr {
    Literal(LiteralArgs),
    Unary(Box<UnaryArgs>),
    Binary(Box<BinaryArgs>),
    Logic(Box<LogicArgs>),
    Grouping(Box<GroupingArgs>),
}

impl Expr {
    pub fn literal(args: LiteralArgs) -> Expr {
        Expr::Literal(args)
    }

    pub fn unary(oper: UnaryOper, expr: Expr) -> Expr {
        Expr::Unary(Box::new(UnaryArgs {
            oper: oper,
            expr: expr,
        }))
    }

    pub fn binary(left: Expr, oper: BinaryOper, right: Expr) -> Expr {
        Expr::Binary(Box::new(BinaryArgs {
            left: left,
            oper: oper,
            right: right,
        }))
    }

    pub fn logic(left: Expr, oper: LogicOper, right: Expr) -> Expr {
        Expr::Logic(Box::new(LogicArgs {
            left: left,
            oper: oper,
            right: right,
        }))
    }

    pub fn group(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(GroupingArgs { expr: expr }))
    }
}

impl From<LiteralArgs> for Expr {
    fn from(item: LiteralArgs) -> Self {
        Expr::Literal(item)
    }
}

pub enum LiteralArgs {
    Nil,
    Bool(bool),
    StringL(String),
    Number(f64),
}

impl From<f64> for LiteralArgs {
    fn from(item: f64) -> Self {
        LiteralArgs::Number(item)
    }
}

impl From<String> for LiteralArgs {
    fn from(item: String) -> Self {
        LiteralArgs::StringL(item)
    }
}

impl From<bool> for LiteralArgs {
    fn from(item: bool) -> Self {
        LiteralArgs::Bool(item)
    }
}

pub struct UnaryArgs {
    pub oper: UnaryOper,
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOper {
    Not,
    Minus,
}

impl From<Token> for Option<UnaryOper> {
    fn from(item: Token) -> Self {
        use Token::*;
        Some(match item {
            Bang => UnaryOper::Not,
            Minus => UnaryOper::Minus,
            _ => panic!("Panic in UnaryOper.from_token"),
        })
    }
}

pub struct BinaryArgs {
    pub left: Expr,
    pub oper: BinaryOper,
    pub right: Expr,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOper {
    Minus,
    Plus,
    Div,
    Mul,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl From<Token> for Option<BinaryOper> {
    fn from(item: Token) -> Self {
        use Token::*;
        Some(match item {
            Minus => BinaryOper::Minus,
            Plus => BinaryOper::Plus,
            Star => BinaryOper::Mul,
            Slash => BinaryOper::Div,
            EqualEqual => BinaryOper::Equal,
            BangEqual => BinaryOper::NotEqual,
            Less => BinaryOper::Less,
            LessEqual => BinaryOper::LessEqual,
            Greater => BinaryOper::Greater,
            GreaterEqual => BinaryOper::GreaterEqual,
            _ => return None,
        })
    }
}

pub struct LogicArgs {
    pub left: Expr,
    pub oper: LogicOper,
    pub right: Expr,
}

#[derive(Debug, Clone, Copy)]
pub enum LogicOper {
    Or,
    And,
}

impl From<Token> for Option<LogicOper> {
    fn from(item: Token) -> Self {
        use Token::*;
        Some(match item {
            Or => LogicOper::Or,
            And => LogicOper::And,
            _ => return None,
        })
    }
}

pub struct GroupingArgs {
    pub expr: Expr,
}
