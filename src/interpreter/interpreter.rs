use super::env::Environment;
use super::visitor::{ExprVisitor, StmtVisitor};
use crate::abs::expr::*;
use crate::abs::stmt::*;
use crate::interpreter::evaluate::EvalExpr;
use crate::interpreter::obj::LoxObj;

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

fn stringify_value(obj: &LoxObj) -> String {
    if let LoxObj::Value(lit) = obj {
        use LiteralArgs::*;
        return match lit {
            Nil => "<nil>".to_string(),
            Bool(b) => b.to_string(),
            // TODO: avoid cloning?
            StringL(s) => s.clone(),
            Number(n) => n.to_string(),
        };
    } else {
        return "ERROR".to_string();
    }
}

// interprete functions
impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        let v = self.eval_expr(expr)?;
        println!("expr: {:?}", v);
        // println!("{:?}", expr);
        Ok(())
    }

    fn visit_print(&mut self, print: &PrintArgs) -> Result<()> {
        let v = self.eval_expr(&print.expr)?;
        println!("print: {:?}", stringify_value(&v));
        Ok(())
    }

    fn visit_var(&mut self, var: &VarDecArgs) -> Result<()> {
        let name = &var.name;
        let v = self.eval_expr(&var.init)?;
        println!("var_dec: {:?} = {:?}", name, v);
        Ok(())
    }
}

fn runtime_err() {
    // line, column, error message
}
