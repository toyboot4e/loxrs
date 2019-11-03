use crate::ast::{expr::*, stmt::*, ExprVisitor, StmtVisitor};
use ::std::collections::HashMap;

// TODO: consider using macros to implement Resolver

type Result<T> = ::std::result::Result<T, SemantcicError>;

#[derive(Debug)]
pub enum SemantcicError {
    // TODO: reporst soure position
    Undefined(String),
    // TODO: separate recursive declaration error
    DuplicateDeclaration(String),
    // TODO: better context (consider assining to tuple with pattern match)
    RecursiveVariableDeclaration(String),
    ReturnFromNonFunction,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LoxFnType {
    None,
    Fn,
    Method,
}

type Scope = HashMap<String, bool>;
// TODO: map id
type Caches = HashMap<VarUseData, usize>;

/// Tracks objects in local scope, analizes them and provides a way to map each variable usage
/// to specific variable in AST.
///
/// It was first introduced for closures.
pub struct Resolver<'a> {
    /// Each scope maps variables to whether it's already initialzied or not.
    /// Useful to detect recursive variable definition or duplicates.
    scopes: Vec<Scope>,
    /// State for function resolving.
    current_fn_type: LoxFnType,
    /// Distances from a scope where each variable is in. Only tracks local variables (see 11.3.2
    /// for details)
    // TODO: isize vs usize
    caches: &'a mut Caches,
}

// TODO: consider returning multiple errors
impl<'a> Resolver<'a> {
    pub fn new(caches: &'a mut Caches) -> Self {
        Self {
            // We don't track global definitions.
            scopes: Vec::new(),
            current_fn_type: LoxFnType::None,
            caches: caches,
        }
    }

    /// Enables to map a local variable to a scope providing the distance to it
    fn resolve_local_var(&mut self, var: &VarUseData) {
        if let Some(d) = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(d, s)| s.contains_key(&var.name))
            .map(|(d, s)| d)
        {
            self.caches.insert(var.clone(), d);
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
    /// Returns error if it finds duplicates.
    fn declare(&mut self, name: &str) -> Result<()> {
        if self.scopes.len() == 0 {
            return Ok(()); // we don't track global variables (see 11.3.2 of the book for details)
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name) {
            return Err(SemantcicError::DuplicateDeclaration(name.to_string()));
        }
        scope.insert(name.to_string(), false);
        Ok(())
    }

    /// States that the item is initialized. Panics if it's not declared.
    fn define(&mut self, name: &str) {
        if self.scopes.len() == 0 {
            return; // we don't track global variables (see 11.3.2 of the book for details)
        }
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), true);
    }

    /// Implemented with Visitor pattern
    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
    }

    /// Implemented with Visitor pattern
    pub fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        self.visit_expr(expr)
    }

    /// Resolves statements in a inner scope
    fn resolve_block(&mut self, stmts: &[Stmt]) -> Result<()> {
        self.begin_scope();
        let result = self.resolve_stmts(stmts);
        self.end_scope();
        result
    }

    /// Just resolves statements
    pub fn resolve_stmts(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn resolve_fn(&mut self, f: &FnDeclArgs, fn_type: LoxFnType) -> Result<()> {
        // tracking state
        let enclosing = self.current_fn_type;
        self.current_fn_type = fn_type;

        self.begin_scope();
        let result = self.impl_resolve_fn(f);
        self.end_scope();

        self.current_fn_type = enclosing;
        result
    }

    /// Resolves function arguments and the body
    fn impl_resolve_fn(&mut self, f: &FnDeclArgs) -> Result<()> {
        if let Some(ref params) = f.params {
            for param in params {
                self.declare(param)?;
                self.define(param);
            }
        }
        self.resolve_stmts(&f.body.stmts)
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_var_decl(&mut self, var: &VarDeclArgs) -> Result<()> {
        self.declare(&var.name)?;
        self.resolve_expr(&var.init)?; // we don't allow to recursively referring to itself
        self.define(&var.name);
        Ok(())
    }

    fn visit_fn_decl(&mut self, f: &FnDeclArgs) -> Result<()> {
        self.declare(&f.name)?;
        self.define(&f.name); // we allow to recursively referring to itself
        self.resolve_fn(f, LoxFnType::Fn)
    }

    // the rest is just passing each stmt/expr to the resolving methods

    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.resolve_expr(expr)
    }

    fn visit_if_stmt(&mut self, if_: &IfArgs) -> Result<()> {
        self.resolve_expr(&if_.condition)?;
        self.resolve_stmt(&if_.if_true)?;
        if let Some(ref if_false) = if_.if_false {
            self.resolve_stmt(if_false)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, print: &PrintArgs) -> Result<()> {
        self.resolve_expr(&print.expr)
    }

    fn visit_return_stmt(&mut self, ret: &Return) -> Result<()> {
        if self.current_fn_type == LoxFnType::None {
            return Err(SemantcicError::ReturnFromNonFunction);
        }
        self.resolve_expr(&ret.expr)
    }

    fn visit_while_stmt(&mut self, while_: &WhileArgs) -> Result<()> {
        self.resolve_expr(&while_.condition)?;
        self.resolve_block(&while_.block.stmts)
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Result<()> {
        self.resolve_block(stmts)
    }

    fn visit_class_decl(&mut self, c: &ClassDeclArgs) -> Result<()> {
        // Lox permits to declare a class as a local variable
        self.declare(&c.name)?;
        self.define(&c.name);
        for m in c.methods.iter() {
            self.resolve_fn(m, LoxFnType::Method)?;
        }
        Ok(())
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_var_expr(&mut self, var: &VarUseData) -> Result<()> {
        // we forbid recursive variable declaration
        if let Some(scope) = self.scopes.last() {
            if scope.get(&var.name) == Some(&false) {
                // cannot read variable in its own initializer
                return Err(SemantcicError::RecursiveVariableDeclaration(
                    var.name.to_string(),
                ));
            }
        }
        self.resolve_local_var(var);
        Ok(())
    }

    fn visit_assign_expr(&mut self, assign: &AssignData) -> Result<()> {
        self.resolve_expr(&assign.expr)?;
        self.resolve_local_var(&assign.assigned);
        Ok(())
    }

    // the rest is just passing each expr to the resolving method

    fn visit_binary_expr(&mut self, binary: &BinaryData) -> Result<()> {
        self.resolve_expr(&binary.left)?;
        self.resolve_expr(&binary.right)
    }

    fn visit_call_expr(&mut self, call: &CallData) -> Result<()> {
        self.resolve_expr(&call.callee)?;
        if let Some(ref args) = call.args {
            for arg in args {
                self.resolve_expr(arg)?;
            }
        }
        Ok(())
    }

    fn visit_logic_expr(&mut self, logic: &LogicData) -> Result<()> {
        self.resolve_expr(&logic.left)?;
        self.resolve_expr(&logic.right)
    }

    fn visit_unary_expr(&mut self, unary: &UnaryData) -> Result<()> {
        self.resolve_expr(&unary.expr)
    }

    fn visit_literal_expr(&mut self, literal: &LiteralData) -> Result<()> {
        Ok(()) // there's no variable mentioned
    }

    fn visit_get_expr(&mut self, get: &GetUseData) -> Result<()> {
        self.resolve_expr(&get.body)
    }

    fn visit_set_expr(&mut self, set: &SetUseData) -> Result<()> {
        self.resolve_expr(&set.body)?;
        self.resolve_expr(&set.value)
    }
}
