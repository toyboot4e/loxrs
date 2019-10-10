use ::std::collections::HashMap;
use crate::runtime::{env::Env, StmtVisitor};
use crate::ast::{expr::*, stmt::*};

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

#[derive(Debug)]
pub enum SemantcicError {
    // TODO: reporst soure position
    Undefined(String),
}

pub struct Resolver {
    /// Each scope maps variable names to if it's already initialized in
    /// the scope.
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn after_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        let mut scope = match self.scopes.pop() {
            None => return,
            Some(s) => s,
        };
        scope.insert(name.to_string(), false);
    }

    fn define(&mut self, name: &str) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), true);
    }

    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    /// Implemented with Visitor pattern
    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
    }
}

impl StmtVisitor<Result<()>> for Resolver {
    fn visit_var_decl(&mut self, var: &VarDecArgs) -> Result<()> {
        unimplemented!()
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

