/// Automates double dispatches reducing `switch`
///
use super::obj::LoxObj;
use crate::abs::expr::*;
use crate::abs::stmt::*;

/// Automates double dispatches reducing `switch`
pub trait ExprVisitor<T> {
    /// Dispathes specific sub function to Expr variants.
    fn visit_expr(&mut self, expr: &Expr) -> T {
        use Expr::*;
        match expr {
            // use as_ref to for unboxing
            Literal(args) => self.visit_literal(args),
            Unary(args) => self.visit_unary(args.as_ref()),
            Binary(args) => self.visit_binary(args.as_ref()),
            Logic(args) => self.visit_logic(args.as_ref()),
            Grouping(args) => self.visit_expr(&args.expr),
            Variable(name) => self.visit_var(name),
        }
    }
    fn visit_literal(&mut self, literal: &LiteralArgs) -> T;
    fn visit_unary(&mut self, unary: &UnaryArgs) -> T;
    fn visit_binary(&mut self, binary: &BinaryArgs) -> T;
    fn visit_logic(&mut self, logic: &LogicArgs) -> T;
    fn visit_var(&mut self, name: &str) -> T;
}

/// Automates double dispatches
pub trait StmtVisitor<T> {
    /// Dispathes specific sub function to Stmt variants
    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        use Stmt::*;
        match stmt {
            Expr(expr) => self.visit_expr_stmt(expr),
            Print(print) => self.visit_print(print),
            Var(var) => self.visit_var_dec(var),
            If(if_) => self.visit_if(if_),
            Block(block) => self.visit_block(block.as_ref()),
        }
    }
    fn visit_var_dec(&mut self, var: &VarDecArgs) -> T;
    fn visit_expr_stmt(&mut self, expr: &Expr) -> T;
    fn visit_print(&mut self, print: &PrintArgs) -> T;
    fn visit_if(&mut self, if_: &IfArgs) -> T;
    fn visit_block(&mut self, block: &[Stmt]) -> T;
}
