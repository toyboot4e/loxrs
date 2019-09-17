use crate::lexer::parser::ParseError;
use crate::lexer::token::Token;
use std::convert::From;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(LiteralArgs),
    Unary(Box<UnaryArgs>),
    Binary(Box<BinaryArgs>),
    Logic(Box<LogicArgs>),
    Grouping(Box<GroupingArgs>),
    Variable(String),
    Assign(Box<AssignArgs>),
    Call(Box<CallArgs>),
}

/// Helpers for constructing / right recursive parsing
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

    /// comparison, addition, or multiplication
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

    pub fn var(name: &str) -> Expr {
        Expr::Variable(name.to_string())
    }

    pub fn assign(left: Expr, oper: AssignOper, right: Expr) -> Result<Expr, ParseError> {
        match left {
            Expr::Variable(ref name) => Ok(Expr::Assign(Box::new(AssignArgs {
                name: name.to_string(),
                expr: right,
            }))),
            _ => Err(ParseError::NotAssignable(left)),
        }
    }

    pub fn call(callee: Expr, args: Option<Args>) -> Self {
        Expr::Call(Box::new(CallArgs {
            callee: callee,
            args: args
        }))
    }
}

impl From<LiteralArgs> for Expr {
    fn from(item: LiteralArgs) -> Self {
        Expr::Literal(item)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LiteralArgs {
    Nil,
    Bool(bool),
    StringLit(String),
    Number(f64),
}

impl LiteralArgs {
    pub fn from_token(token: &Token) -> Option<LiteralArgs> {
        use Token::*;
        Some(match token {
            Nil => LiteralArgs::Nil,
            True => LiteralArgs::Bool(true),
            False => LiteralArgs::Bool(false),
            String(ref s) => LiteralArgs::StringLit(s.clone()),
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
        LiteralArgs::StringLit(item)
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

impl BinaryOper {
    pub fn is_logic(&self) -> bool {
        use BinaryOper::*;
        match self {
            Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual => true,
            _ => false,
        }
    }
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

/// `&&` or `||`
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

/// `()`
#[derive(Clone, Debug, PartialEq)]
pub struct GroupingArgs {
    pub expr: Expr,
}

/// `=`,  only parsed as an expression statement.
#[derive(Clone, Debug, PartialEq)]
pub struct AssignArgs {
    /// Name of the identifier to assign
    pub name: String,
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AssignOper {
    Equal,
}

impl From<Token> for Option<AssignOper> {
    fn from(item: Token) -> Self {
        use Token::*;
        Some(match item {
            Equal => AssignOper::Equal,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallArgs {
    pub callee: Expr,
    pub args: Option<Args>,
}

pub type Args = Vec<Expr>;
