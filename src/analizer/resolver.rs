use crate::ast::{expr::*, stmt::*};
use crate::runtime::{env::Env, ExprVisitor, StmtVisitor};
use ::std::collections::HashMap;

type Result<T> = ::std::result::Result<T, SemantcicError>;

#[derive(Debug)]
pub enum SemantcicError {
    // TODO: reporst soure position
    Undefined(String),
    // TODO: separate recursive declaration error
    DuplicateVariableDeclaration(String),
    ReturnFromNonFunction,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FnType {
    None,
    Fn,
}

type Locals = HashMap<Expr, usize>;

pub struct Resolver<'a> {
    /// Nested scopes which maps variable names to whether it's already initialized.
    scopes: Vec<HashMap<String, bool>>,
    /// State to forbid returning fron non-function
    current_fn_type: FnType,
    /// Caches for the result of resolving
    // TODO: isize vs usize
    locals: &'a mut Locals,
}

impl<'a> Resolver<'a> {
    pub fn new(locals: &'a mut Locals) -> Self {
        Self {
            scopes: Vec::new(),
            current_fn_type: FnType::None,
            locals: locals,
        }
    }

    /// Resolves an expression. To be called only from `resolve_local_var`.
    fn impl_resolve(&mut self, expr: &Expr, d: usize) {
        self.locals.insert(expr.clone(), d);
    }

    fn resolve_local_var(&mut self, expr: &Expr, name: &str) {
        if let Some(d) = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(d, s)| s.contains_key(name))
            .map(|(d, s)| d)
        {
            // finally, actually resolve the expression
            self.impl_resolve(expr, d);
        }
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

    /// Implemented with Visitor pattern
    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
    }

    /// Implemented with visitor pattern
    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    /// Resolve the function definition updating the `Resolver`'s state
    pub fn resolve_fn(&mut self, f: &FnDef, type_: FnType) -> Result<()> {
        let enclosing = self.current_fn_type;
        self.current_fn_type = type_;

        let result = self.impl_resolve_fn(f);

        self.current_fn_type = enclosing;
        result
    }

    fn impl_resolve_fn(&mut self, f: &FnDef) -> Result<()> {
        self.begin_scope();
        if let Some(ref params) = f.params {
            for param in params {
                self.declare(param);
                self.define(param);
            }
        }
        self.resolve_stmts(&f.body.stmts)?;
        self.end_scope();
        Ok(())
    }

    /// Implemented with Visitor pattern
    pub fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        self.visit_expr(expr)
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_var_decl(&mut self, var: &VarDecArgs) -> Result<()> {
        self.declare(&var.name); // we forbid recursive variable declaration
        self.resolve_local_var(&var.init, &var.name);
        self.define(&var.name); // now it's initialized
        Ok(())
    }

    fn visit_fn_decl(&mut self, f: &FnDef) -> Result<()> {
        self.declare(&f.name);
        self.define(&f.name); // we allow recursive function declaration
        self.resolve_fn(f, FnType::Fn)?;
        Ok(())
    }

    // the rest is just passing each stmt/expr to the resolving methods

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.resolve_expr(expr)?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, if_: &IfArgs) -> Result<()> {
        self.resolve_expr(&if_.condition)?;
        self.resolve_stmt(&if_.if_true)?;
        if let Some(ref else_branch) = if_.if_false {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, print: &PrintArgs) -> Result<()> {
        self.resolve_expr(&print.expr)?;
        Ok(())
    }

    fn visit_return_stmt(&mut self, ret: &Return) -> Result<()> {
        if self.current_fn_type == FnType::None {
            return Err(SemantcicError::ReturnFromNonFunction);
        }
        self.resolve_expr(&ret.expr)?;
        Ok(())
    }

    fn visit_while_stmt(&mut self, while_: &WhileArgs) -> Result<()> {
        self.resolve_expr(&while_.condition)?;
        self.resolve_stmts(&while_.block.stmts)?;
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>, env: Option<Env>) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_var_expr(&mut self, name: &str) -> Result<()> {
        // we forbid duplicate variable declaration
        if let Some(scope) = self.scopes.last() {
            if scope.contains_key(name) {
                return Err(SemantcicError::DuplicateVariableDeclaration(
                    name.to_string(),
                ));
            }
        }
        // TODO: consider containg Expr in the signature
        let expr 
        self.resolve_local_var(name);
        Ok(())
    }

    // the rest is just passing each expr to the resolving method

    fn visit_binary_expr(&mut self, binary: &BinaryArgs) -> Result<()> {
        self.resolve_expr(&binary.left)?;
        self.resolve_expr(&binary.right)?;
        Ok(())
    }

    fn visit_call_expr(&mut self, call: &CallArgs) -> Result<()> {
        if let Some(ref args) = call.args {
            for arg in args {
                self.resolve_expr(arg)?;
            }
        }
        Ok(())
    }

    fn visit_logic_expr(&mut self, logic: &LogicArgs) -> Result<()> {
        self.resolve_expr(&logic.left)?;
        self.resolve_expr(&logic.right)?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, unary: &UnaryArgs) -> Result<()> {
        self.resolve_expr(&unary.expr)?;
        Ok(())
    }

    fn visit_literal_expr(&mut self, literal: &LiteralArgs) -> Result<()> {
        unimplemented!()
    }

    fn visit_assign_expr(&mut self, assign: &AssignArgs) -> Result<()> {
        unimplemented!()
    }
}
