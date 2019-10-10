use crate::ast::{expr::*, stmt::*};
use crate::runtime::{env::Env, ExprVisitor, StmtVisitor};
use ::std::collections::HashMap;

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

#[derive(Debug)]
pub enum SemantcicError {
    // TODO: reporst soure position
    Undefined(String),
    // TODO: separate recursive declaration error
    DuplicateVariableDeclaration(String),
}

pub struct Resolver {
    /// Each scope maps variable names to if it's already initialized in
    /// the scope.
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Creates new scope
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Removes the innermost scope
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    /// States that the item exists but not initialized yet.
    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        };
    }

    /// States that the item is initialized. Panics if it's not declared.
    fn define(&mut self, name: &str) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), true);
    }

    fn resolve_local_var(&mut self, name: &str, init: &Expr) {
        if let Some(i) = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(i, s)| s.contains_key(name))
            .map(|(i, s)| i)
        {
            unimplemented!()
        }
    }

    /// Implemented with Visitor pattern
    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
    }

    /// implemented with visitor pattern
    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    /// Implemented with Visitor pattern
    pub fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        self.visit_expr(expr)
    }
}

impl StmtVisitor<Result<()>> for Resolver {
    fn visit_var_decl(&mut self, var: &VarDecArgs) -> Result<()> {
        self.declare(&var.name);
        self.resolve_local_var(&var.name, &var.init); // we forbid recursive variable declaration
        self.define(&var.name);
        Ok(())
    }

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        unimplemented!()
    }

    fn visit_print_stmt(&mut self, print: &PrintArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_if_stmt(&mut self, if_: &IfArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>, env: Option<Env>) -> Result<()> {
        unimplemented!()
    }

    fn visit_return_stmt(&mut self, ret: &Return) -> Result<()> {
        unimplemented!()
    }

    fn visit_while_stmt(&mut self, while_: &WhileArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_fn_decl(&mut self, f: &FnDef) -> Result<()> {
        unimplemented!()
    }
}

impl ExprVisitor<Result<()>> for Resolver {
    fn visit_var_expr(&mut self, name: &str) -> Result<()> {
        // we forbid duplicate variable declaration
        if let Some(scope) = self.scopes.last() {
            if scope.contains_key(name) {
                return Err(SemantcicError::DuplicateVariableDeclaration(name.to_string()));
            }
        }
        self.resolve_local_var(var.name, var.init);
        Ok(())
    }

    fn visit_literal_expr(&mut self, literal: &LiteralArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_unary_expr(&mut self, unary: &UnaryArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_binary_expr(&mut self, binary: &BinaryArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_logic_expr(&mut self, logic: &LogicArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_assign_expr(&mut self, assign: &AssignArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_call_expr(&mut self, call: &CallArgs) -> Result<()> {
        unimplemented!()
    }
}
