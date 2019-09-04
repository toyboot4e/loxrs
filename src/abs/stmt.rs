use crate::abs::expr::Expr;

// FIXME: where to box
/// Stmt → expr | if | print | block ;
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    /// exprStmt  → expression ";" ;
    Expr(Expr),
    /// printStmt → "print" expression ";" ;
    Print(PrintArgs),
    Var(VarDecArgs),
    If(Box<IfArgs>),
    Block(Vec<Stmt>),
}

impl Stmt {
    pub fn expr(expr: Expr) -> Self {
        Stmt::Expr(expr)
    }

    pub fn print(expr: Expr) -> Self {
        Stmt::Print(PrintArgs { expr: expr })
    }

    pub fn var_dec(name: String, init: Expr) -> Self {
        Stmt::Var(VarDecArgs::new(name, init))
    }

    pub fn if_then_else(condition: Expr, then: Stmt, else_: Option<Stmt>) -> Self {
        Stmt::If(Box::new(IfArgs {
            condition: condition,
            if_true: then,
            if_false: else_,
        }))
    }
}

impl From<PrintArgs> for Stmt {
    fn from(item: PrintArgs) -> Self {
        Stmt::Print(item)
    }
}

impl From<VarDecArgs> for Stmt {
    fn from(item: VarDecArgs) -> Self {
        Stmt::Var(item)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintArgs {
    // pub message: String,
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarDecArgs {
    pub name: String,
    pub init: Expr,
}

// split into IfThen and IfThenElse
#[derive(Clone, Debug, PartialEq)]
pub struct IfArgs {
    pub condition: Expr,
    pub if_true: Stmt,
    /// May be `if`
    pub if_false: Option<Stmt>,
}

impl VarDecArgs {
    /// Unlike the original Lox language, loxrs always requires initializer for declarations
    pub fn new(name: String, init: Expr) -> Self {
        Self {
            name: name,
            init: init,
        }
    }
}
