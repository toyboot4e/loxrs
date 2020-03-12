use crate::lexer::token::Token;
use std::convert::From;

// We need to make `Expr` hashable so that we can map `Expr` to distance
// in `Resolver`.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(LiteralData),
    Unary(Box<UnaryData>),
    Binary(Box<BinaryData>),
    Logic(Box<LogicData>),
    Grouping(Box<GroupData>),
    Variable(VarUseData),
    /// Assignment to a variable
    Assign(Box<AssignData>),
    Call(Box<CallData>),
    /// Solves a scope or field
    Get(Box<GetUseData>),
    // Assignment to a field of an instance
    Set(Box<SetUseData>),
    Self_(SelfData),
}

/// Helpers for constructing / right recursive parsing
impl Expr {
    pub fn literal(args: LiteralData) -> Expr {
        Expr::Literal(args)
    }

    pub fn unary(oper: UnaryOper, expr: Expr) -> Expr {
        Expr::Unary(Box::new(UnaryData {
            oper: oper,
            expr: expr,
        }))
    }

    /// comparison, addition, or multiplication
    pub fn binary(left: Expr, oper: BinaryOper, right: Expr) -> Expr {
        Expr::Binary(Box::new(BinaryData {
            left: left,
            oper: oper,
            right: right,
        }))
    }

    pub fn logic(left: Expr, oper: LogicOper, right: Expr) -> Expr {
        Expr::Logic(Box::new(LogicData {
            left: left,
            oper: oper,
            right: right,
        }))
    }

    pub fn group(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(GroupData { expr: expr }))
    }

    pub fn var(name: &str, id: VarUseId) -> Expr {
        Expr::Variable(VarUseData::new(name, id))
    }

    /// Assignment to a variable
    pub fn assign(name: &str, expr: Expr, id: VarUseId) -> Expr {
        Expr::Assign(Box::new(AssignData {
            assigned: VarUseData::new(name, id),
            expr: expr,
        }))
    }

    /// Assignment to a field of an instance
    pub fn set(body: Expr, name: &str, value: Expr) -> Expr {
        Expr::Set(Box::new(SetUseData::new(body, name, value)))
    }

    pub fn get(body: Expr, name: &str) -> Expr {
        Expr::Get(Box::new(GetUseData::new(body, name)))
    }

    pub fn call(callee: Expr, args: Args) -> Self {
        Expr::Call(Box::new(CallData {
            callee: callee,
            args: args,
        }))
    }
}

impl From<LiteralData> for Expr {
    fn from(item: LiteralData) -> Self {
        Expr::Literal(item)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LiteralData {
    Nil,
    Bool(bool),
    StringLit(String),
    Number(f64),
}

impl LiteralData {
    /// Maps specific tokens to `Option::Some(LiteralData)`
    pub fn from_token(token: &Token) -> Option<LiteralData> {
        use Token::*;
        Some(match token {
            Nil => LiteralData::Nil,
            True => LiteralData::Bool(true),
            False => LiteralData::Bool(false),
            String(ref s) => LiteralData::StringLit(s.clone()),
            Number(n) => LiteralData::Number(n.clone()),
            _ => return None,
        })
    }
}

// They are convenient for writing tests.
impl From<f64> for LiteralData {
    fn from(item: f64) -> Self {
        LiteralData::Number(item)
    }
}

impl From<String> for LiteralData {
    fn from(item: String) -> Self {
        LiteralData::StringLit(item)
    }
}

impl From<bool> for LiteralData {
    fn from(item: bool) -> Self {
        LiteralData::Bool(item)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryData {
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
pub struct BinaryData {
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
pub struct LogicData {
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
pub struct GroupData {
    pub expr: Expr,
}

/// Enables to track each variable use. It's required by the `Resolver`.
///
/// We might be able to use source position instead, but my AST doesn't track that information.
/// So I embeded ID in AST.
// TODO: refactor when I add more context to error information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarUseId {
    id: usize,
}

impl VarUseId {
    pub fn new() -> Self {
        Self { id: 0 }
    }
}

/// Creates new ID.
pub struct VarUseIdCounter {
    id: usize,
}

impl VarUseIdCounter {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    pub fn next(&mut self) -> VarUseId {
        self.id += 1;
        VarUseId { id: self.id - 1 }
    }
}

/// Represents a variable use
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VarUseData {
    pub name: String,
    /// Unique identity of each variable use
    pub id: VarUseId,
}

impl VarUseData {
    pub fn new(name: &str, id: VarUseId) -> Self {
        Self {
            name: name.to_string(),
            id: id,
        }
    }
}

/// `=`,  only parsed as an expression statement.
///
/// It doesn't contain LHS object 'cause. Instead, it should be gotten from `Env`.
#[derive(Clone, Debug, PartialEq)]
pub struct AssignData {
    pub assigned: VarUseData,
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

pub type Args = Vec<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub struct CallData {
    pub callee: Expr,
    // FIXME: just use `Args` type
    pub args: Args,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GetUseData {
    pub body: Expr,
    pub name: String,
}

impl GetUseData {
    pub fn new(body: Expr, name: &str) -> Self {
        Self {
            body: body,
            name: name.to_string(),
        }
    }
}

/// It's similar to an assignment, but it tries to assign value to
#[derive(Clone, Debug, PartialEq)]
pub struct SetUseData {
    pub body: Expr,
    pub name: String,
    pub value: Expr,
}

impl SetUseData {
    pub fn new(body: Expr, name: &str, value: Expr) -> Self {
        Self {
            body: body,
            name: name.to_string(),
            value: value,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelfData {}
