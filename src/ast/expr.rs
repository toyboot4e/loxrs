use crate::lexer::token::Token;
use std::convert::From;

// We need to make `Expr` hashable so that we can map `Expr` to distance
// in `Resolver`.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(LiteralArgs),
    Unary(Box<UnaryArgs>),
    Binary(Box<BinaryArgs>),
    Logic(Box<LogicArgs>),
    Grouping(Box<GroupingArgs>),
    // TODO: rename me; it may be function
    Variable(VariableArgs),
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

    pub fn var(name: &str, id: VarUseId) -> Expr {
        Expr::Variable(VariableArgs::new(name, id))
    }

    pub fn assign(name: &str, expr: Expr, id: VarUseId) -> Expr {
        Expr::Assign(Box::new(AssignArgs {
            assigned: VariableArgs::new(name, id),
            expr: expr,
        }))
    }

    pub fn call(callee: Expr, args: Option<Args>) -> Self {
        Expr::Call(Box::new(CallArgs {
            callee: callee,
            args: args,
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
    /// Maps specific tokens to `Option::Some(LiteralArgs)`
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
pub struct VariableArgs {
    pub name: String,
    /// Unique identity of each variable use
    pub id: VarUseId,
}

impl VariableArgs {
    pub fn new(name: &str, id: VarUseId) -> Self {
        Self {
            name: name.to_string(),
            id: id,
        }
    }
}

/// `=`,  only parsed as an expression statement.
#[derive(Clone, Debug, PartialEq)]
pub struct AssignArgs {
    pub assigned: VariableArgs,
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
pub struct CallArgs {
    pub callee: Expr,
    pub args: Option<Args>,
}
