//! Automates double dispatches reducing `switch`

use crate::ast::{expr::*, stmt::*};
use crate::runtime::env::Env;

/// Automates double dispatches reducing `switch`
pub trait ExprVisitor<T> {
    /// Dispathes specific sub function to Expr variants.
    fn visit_expr(&mut self, expr: &Expr) -> T {
        use Expr::*;
        match expr {
            // use as_ref to for unboxing
            Literal(args) => self.visit_literal_expr(args),
            Unary(args) => self.visit_unary_expr(args.as_ref()),
            Binary(args) => self.visit_binary_expr(args.as_ref()),
            Logic(args) => self.visit_logic_expr(args.as_ref()),
            Grouping(args) => self.visit_expr(&args.expr),
            Variable(name) => self.visit_var_expr(name),
            Assign(args) => self.visit_assign_expr(args.as_ref()),
            Call(call) => self.visit_call_expr(call.as_ref()),
        }
    }
    fn visit_literal_expr(&mut self, literal: &LiteralArgs) -> T;
    fn visit_unary_expr(&mut self, unary: &UnaryArgs) -> T;
    fn visit_binary_expr(&mut self, binary: &BinaryArgs) -> T;
    fn visit_logic_expr(&mut self, logic: &LogicArgs) -> T;
    fn visit_var_expr(&mut self, name: &str) -> T;
    fn visit_assign_expr(&mut self, assign: &AssignArgs) -> T;
    fn visit_call_expr(&mut self, call: &CallArgs) -> T;
}

/// Automates double dispatches
pub trait StmtVisitor<T> {
    /// Dispathes specific sub function to Stmt variants
    fn visit_stmt(&mut self, stmt: &Stmt) -> T {
        use Stmt::*;
        match stmt {
            Expr(expr) => self.visit_expr_stmt(expr),
            Print(print) => self.visit_print_stmt(print),
            Var(var) => self.visit_var_decl(var),
            If(if_) => self.visit_if_stmt(if_),
            Block(block) => self.visit_block_stmt(block.stmts.as_ref(), None),
            Return(ret) => self.visit_return_stmt(ret),
            While(while_) => self.visit_while_stmt(while_),
            Fn(f) => self.visit_fn_decl(f),
        }
    }
    fn visit_var_decl(&mut self, var: &VarDecArgs) -> T;
    /// Expression statements for side effects
    fn visit_expr_stmt(&mut self, expr: &Expr) -> T;
    /// Built-in print statement (not a function)
    // TODO: make `print` a native function
    fn visit_print_stmt(&mut self, print: &PrintArgs) -> T;
    fn visit_if_stmt(&mut self, if_: &IfArgs) -> T;
    /// We need local scope for function blocks
    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>, env: Option<Env>) -> T;
    fn visit_return_stmt(&mut self, ret: &Return) -> T;
    fn visit_while_stmt(&mut self, while_: &WhileArgs) -> T;
    // TODO: disable clock as a variable name? (or distinguish two scopes like Lisp 2?)
    fn visit_fn_decl(&mut self, f: &FnDef) -> T;
}
