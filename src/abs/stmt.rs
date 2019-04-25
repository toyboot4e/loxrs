use crate::abs::expr::Expr;

// exprStmt  → expression ";" ;
// printStmt → "print" expression ";" ;
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Print(Box<PrintArgs>),
    Var(Box<VarDecArgs>),
}

impl Stmt {
    pub fn expr(expr: Expr) -> Self {
        Stmt::Expr(Box::new(expr))
    }

    pub fn print(message: String) -> Self {
        Stmt::Print(Box::new(PrintArgs { message: message }))
    }

    pub fn var_dec(name: String, init: Expr) -> Self {
        Stmt::Var(Box::new(VarDecArgs::new(name, init)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintArgs {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarDecArgs {
    pub name: String,
    pub init: Box<Expr>,
}

impl VarDecArgs {
    /// Unlike the original Lox language, initializer is always explicit.
    pub fn new(name: String, init: Expr) -> Self {
        Self {
            name: name,
            init: Box::new(init),
        }
    }
}
