use crate::abs::expr::Expr;

// exprStmt  → expression ";" ;
// printStmt → "print" expression ";" ;
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Print(PrintArgs),
    Var(Box<VarDecArgs>),
    Block(Vec<Stmt>),
}

impl Stmt {
    pub fn expr(expr: Expr) -> Self {
        Stmt::Expr(Box::new(expr))
    }

    pub fn print(expr: Expr) -> Self {
        Stmt::Print(PrintArgs { expr: expr })
    }

    pub fn var_dec(name: String, init: Expr) -> Self {
        Stmt::Var(Box::new(VarDecArgs::new(name, init)))
    }
}

impl From<PrintArgs> for Stmt {
    fn from(item: PrintArgs) -> Self {
        Stmt::Print(item)
    }
}

impl From<VarDecArgs> for Stmt {
    fn from(item: VarDecArgs) -> Self {
        Stmt::Var(Box::new(item))
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
    pub init: Box<Expr>,
}

impl VarDecArgs {
    /// Unlike the original Lox language, initializer is always needed.
    pub fn new(name: String, init: Expr) -> Self {
        Self {
            name: name,
            init: Box::new(init),
        }
    }
}
