// specific subtypes are always need for pretty printer.
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

pub enum LiteralArgs {
    Nil,
    Bool(bool),
    StringL(String),
    Number(f64),
}

pub struct UnaryArgs {
    pub oper: UnaryOper,
    pub expr: Expr,
}

#[derive(Debug)]
pub enum UnaryOper {
    Bang,
    Minus,
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
    Slash,
    Star,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

// Same as BinaryExpr, but with short-circuiting
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

pub struct GroupingArgs {
    pub expr: Expr,
}
