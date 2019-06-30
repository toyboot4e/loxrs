use super::obj::LoxObj;
use crate::abs::expr::*;
use crate::abs::stmt::*;

/// Automates double dispatches.
pub trait ExprVisitor<T> {
    /// Dispathes specific sub function
    fn visit_expr(&mut self, expr: &Expr) -> T {
        use Expr::*;
        match expr {
            // use as_ref to for unboxing
            Literal(args) => self.visit_literal(args),
            Unary(args) => self.visit_unary(args.as_ref()),
            Binary(args) => self.visit_binary(args.as_ref()),
            Logic(args) => self.visit_logic(args.as_ref()),
            Grouping(args) => self.visit_expr(&args.expr),
        }
    }
    fn visit_literal(&mut self, literal: &LiteralArgs) -> T;
    fn visit_unary(&mut self, unary: &UnaryArgs) -> T;
    fn visit_binary(&mut self, binary: &BinaryArgs) -> T;
    fn visit_logic(&mut self, logic: &LogicArgs) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        use Stmt::*;
        match stmt {
            Expr(expr) => self.visit_expr(expr.as_ref()),
            Print(print) => self.visit_print(print),
            Var(var) => self.visit_var(var.as_ref()),
        }
    }
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_print(&mut self, print: &PrintArgs) -> T;
    fn visit_var(&mut self, var: &VarDecArgs) -> T;
}
