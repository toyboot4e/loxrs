use crate::abs::token::Token;
use std::convert::From;
// TODO: benchmark lazy static vs match
// TODO: combining oper and token or not

// specific subtypes are always needed for the pretty printer.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(LiteralArgs),
    Unary(Box<UnaryArgs>),
    Binary(Box<BinaryArgs>),
    Logic(Box<LogicArgs>),
    Grouping(Box<GroupingArgs>),
    // Variable(String),
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

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralArgs {
    Nil,
    Bool(bool),
    StringL(String),
    Number(f64),
}

impl LiteralArgs {
    pub fn from_token(t: &Token) -> Option<LiteralArgs> {
        use Token::*;
        Some(match t {
            Nil => LiteralArgs::Nil,
            True => LiteralArgs::Bool(true),
            False => LiteralArgs::Bool(false),
            String(ref s) => LiteralArgs::StringL(s.clone()),
            Number(n) => LiteralArgs::Number(n.clone()),
            _ => return None,
        })
    }
}

// They are convenient for writing tests.
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryArgs {
    pub left: Expr,
    pub oper: BinaryOper,
    pub right: Expr,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct LogicArgs {
    pub left: Expr,
    pub oper: LogicOper,
    pub right: Expr,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct GroupingArgs {
    pub expr: Expr,
}
