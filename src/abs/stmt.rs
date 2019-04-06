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

    pub fn execute(&mut self) {
        use Stmt::*;
        match self {
            Expr(expr) => unimplemented!(),
            Print(print) => print.execute(),
            _ => {}
        }
    }

    pub fn var_dec(name: String, init: Expr) -> Self {
        Stmt::Var(Box::new(VarDecArgs::new(name, init)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintArgs {
    message: String,
}

impl PrintArgs {
    pub fn execute(&self) {
        println!("print: {}", self.message);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarDecArgs {
    name: String,
    init: Box<Expr>,
}

impl VarDecArgs {
    pub fn new(name: String, init: Expr) -> Self {
        Self {
            name: name,
            init: Box::new(init),
        }
    }
}
