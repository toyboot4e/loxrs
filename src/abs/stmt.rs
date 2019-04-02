use super::expr::Expr;
use crate::token::Token;

// exprStmt  → expression ";" ;
// printStmt → "print" expression ";" ;
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Print(Box<PrintArgs>),
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
            Expr(expr) => expr.evaluate(),
            Print(print) => print.execute(),
        }
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
