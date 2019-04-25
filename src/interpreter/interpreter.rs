use super::env::Environment;
use super::visitor::StmtVisitor;
use crate::abs::expr::*;
use crate::abs::stmt::*;

/// Runtime error when evaluating expressions.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    MismatchedType,
}

type Result<T> = ::std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        println!("{:?}", expr);
        Ok(())
    }

    fn visit_print(&mut self, print: &PrintArgs) -> Result<()> {
        println!("print: {}", print.message);
        Ok(())
    }

    fn visit_var(&mut self, var: &VarDecArgs) -> Result<()> {
        unimplemented!();
    }
}

fn runtime_err() {
    // line, column, error message
}
