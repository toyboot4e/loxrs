use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use super::env::Env;
use super::visitor::StmtVisitor;

use crate::ast::PrettyPrint;
use crate::ast::{expr::*, stmt::*};
use crate::runtime::{
    obj::{LoxFn, LoxObj, LoxValue},
    Result, RuntimeError,
};

// TODO: encapsulate `Rc<Refcell<T>>`
pub struct Interpreter {
    /// Points to the global `Env`
    globals: Rc<RefCell<Env>>,
    /// Temporary `Env` for proessing
    pub env: Rc<RefCell<Env>>,
    /// When the interpretation started; enables `clock` native function
    begin_time: SystemTime,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Self::global_env()));
        let env = Rc::new(RefCell::new(Env::from_parent(&globals)));
        Self {
            globals: globals,
            env: env,
            begin_time: SystemTime::now(),
        }
    }

    /// Creates a new `Env` with native functions
    fn global_env() -> Env {
        let mut env = Env::new();
        env.define("clock", LoxObj::Callable(LoxFn::Clock)).unwrap();
        env
    }

    /// The entry point of interpretation
    pub fn interpret(&mut self, stmt: &Stmt) -> Result<Option<LoxObj>> {
        self.visit_stmt(stmt)
    }

    /// Invokes a given function object
    pub fn invoke(&mut self, fn_obj: &LoxFn, args: &Option<Args>) -> Result<Option<LoxObj>> {
        let def = match fn_obj {
            LoxFn::User(ref def) => def,
            LoxFn::Clock => {
                let s = self.native_clock(args)?;
                return Ok(Some(LoxObj::Value(s)));
            }
        };

        Self::validate_arities(
            def.params.as_ref().map(|xs| xs.len()),
            args.as_ref().map(|xs| xs.len()),
        )?;

        // create new environment with the arguments
        let mut new_env = Env::from_parent(&self.env);
        if let (Some(params), Some(args)) = (&def.params, args) {
            for i in 0..params.len() {
                new_env.define(params[i].as_str(), self.eval_expr(&args[i])?)?;
            }
        }

        let result = self.visit_block_stmt(&def.body, Some(new_env))?;
        Ok(result)
    }

    /// Compares two arities (each of which may be None) and makes sure they match
    fn validate_arities(n1: Option<usize>, n2: Option<usize>) -> Result<Option<LoxObj>> {
        match (n1, n2) {
            (None, None) => Ok(None),
            (Some(_), None) | (None, Some(_)) => Err(RuntimeError::WrongNumberOfArguments),
            (Some(len_params), Some(len_args)) if len_params != len_args => {
                Err(RuntimeError::WrongNumberOfArguments)
            }
            _ => Ok(None),
        }
    }

    /// Milli seconds since the Lox program is started
    pub fn native_clock(&self, args: &Option<Args>) -> Result<LoxValue> {
        if !args.is_none() {
            return Err(RuntimeError::WrongNumberOfArguments);
        }
        Ok(LoxValue::Number(
            //self.on_begin.elapsed().unwrap().as_secs() as f64
            self.begin_time.elapsed().unwrap().as_millis() as f64,
        ))
    }
}

fn stringify_obj(obj: &LoxObj) -> String {
    if let LoxObj::Value(lit) = obj {
        use LoxValue::*;
        match lit {
            Nil => "<nil>".to_string(),
            Bool(b) => b.to_string(),
            // TODO: avoid cloning?
            StringLit(s) => s.clone(),
            Number(n) => n.to_string(),
        }
    } else {
        "ERROR WHEN STRINGIFY LOX OBJECT".to_string()
    }
}

/// Implements statement interpretation via Visitor pattern
///
/// If something is returned, it's by `return` so we finish interpreting
impl StmtVisitor<Result<Option<LoxObj>>> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<Option<LoxObj>> {
        let v = self.eval_expr(expr)?;
        Ok(None)
    }

    fn visit_print_stmt(&mut self, print: &PrintArgs) -> Result<Option<LoxObj>> {
        let obj = self.eval_expr(&print.expr)?;
        println!("{}", obj.pretty_print());
        Ok(None)
    }

    fn visit_var_decl(&mut self, var: &VarDecArgs) -> Result<Option<LoxObj>> {
        let name = &var.name;
        let obj = self.eval_expr(&var.init)?;
        self.env.borrow_mut().define(name, obj)?;
        Ok(None)
    }

    fn visit_if_stmt(&mut self, if_: &IfArgs) -> Result<Option<LoxObj>> {
        if self.eval_expr(&if_.condition)?.is_truthy() {
            self.interpret(&if_.if_true)
        } else if let Some(if_false) = if_.if_false.as_ref() {
            self.interpret(if_false)
        } else {
            Ok(None)
        }
    }

    fn visit_block_stmt(
        &mut self,
        stmts: &Vec<Stmt>,
        new_env: Option<Env>,
    ) -> Result<Option<LoxObj>> {
        let new_env = new_env.unwrap_or_else(|| Env::from_parent(&self.env));

        let prev = Rc::clone(&self.env);
        self.env = Rc::new(RefCell::new(new_env));

        for stmt in stmts.iter() {
            // early return considered
            match self.interpret(stmt)? {
                Some(obj) => {
                    self.env = prev;
                    return Ok(Some(obj));
                }
                None => {}
            }
        }
        self.env = prev;
        Ok(None)
    }

    // TODO: enable returning even outside block
    fn visit_return_stmt(&mut self, ret: &Return) -> Result<Option<LoxObj>> {
        let obj = self.eval_expr(&ret.expr)?;
        Ok(Some(obj))
    }

    fn visit_while_stmt(&mut self, while_: &WhileArgs) -> Result<Option<LoxObj>> {
        while self.eval_expr(&while_.condition)?.is_truthy() {
            // early return considered
            self.visit_block_stmt(&while_.block.stmts, None)?;
        }
        Ok(None)
    }

    fn visit_fn_decl(&mut self, def: &FnDef) -> Result<Option<LoxObj>> {
        let f = LoxObj::f(def, &self.env);
        self.env.borrow_mut().define(def.name.as_str(), f)?;
        Ok(None)
    }
}

fn runtime_err() {
    // line, column, error message
}

pub trait EvalExpr {
    /// Dispatches a sub function to a specific Expr
    fn eval_expr(&mut self, expr: &Expr) -> Result<LoxObj>;
}

impl EvalExpr for Interpreter {
    fn eval_expr(&mut self, expr: &Expr) -> Result<LoxObj> {
        self.visit_expr(expr)
    }
}

// TODO: using Value struct
use LoxObj::Value as ValObj;

mod logic {
    //! Operator overloading for specific LoxObj_s.

    use crate::runtime::obj::{LoxObj, LoxValue};
    use std::cmp::Ordering;

    pub fn obj_eq(left: &LoxValue, right: &LoxValue) -> Option<bool> {
        Some(match (left, right) {
            (LoxValue::Number(n1), LoxValue::Number(n2)) => n1 == n2,
            (LoxValue::Bool(b1), LoxValue::Bool(b2)) => b1 == b2,
            (LoxValue::StringLit(s1), LoxValue::StringLit(s2)) => s1 == s2,
            _ => return None,
        })
    }

    pub fn obj_cmp(left: &LoxValue, right: &LoxValue) -> Option<Ordering> {
        match (left, right) {
            (LoxValue::Number(n1), LoxValue::Number(n2)) => n1.partial_cmp(n2),
            _ => None,
        }
    }

    pub fn obj_plus(left: &LoxValue, right: &LoxValue) -> Option<LoxObj> {
        use LoxValue::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 + n2),
            (StringLit(s1), StringLit(s2)) => StringLit(format!("{}{}", s1, s2)),
            _ => return None,
        }))
    }

    pub fn obj_minus(left: &LoxValue, right: &LoxValue) -> Option<LoxObj> {
        use LoxValue::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 - n2),
            _ => return None,
        }))
    }

    pub fn obj_div(left: &LoxValue, right: &LoxValue) -> Option<LoxObj> {
        use LoxValue::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 / n2),
            _ => return None,
        }))
    }

    pub fn obj_mul(left: &LoxValue, right: &LoxValue) -> Option<LoxObj> {
        use LoxValue::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 * n2),
            _ => return None,
        }))
    }
}

use crate::runtime::visitor::ExprVisitor;
use std::cmp::Ordering;

/// Visitors for implementing `eval_expr`
impl ExprVisitor<Result<LoxObj>> for Interpreter {
    fn visit_literal_expr(&mut self, lit: &LiteralArgs) -> Result<LoxObj> {
        Ok(ValObj(LoxValue::from_lit(lit)))
    }

    fn visit_unary_expr(&mut self, unary: &UnaryArgs) -> Result<LoxObj> {
        let obj = self.visit_expr(&unary.expr)?;
        use UnaryOper::*;
        match &unary.oper {
            Minus => {
                let n = obj.as_num().ok_or_else(|| RuntimeError::MismatchedType)?;
                Ok(LoxObj::Value(LoxValue::Number(-n)))
            }
            Not => Ok(LoxObj::bool(!obj.is_truthy())),
        }
    }

    /// `==`, `!=`, `<`, `<=`, `>`, `>=`, `+`, `-`, `*`, `/`
    fn visit_binary_expr(&mut self, binary: &BinaryArgs) -> Result<LoxObj> {
        use BinaryOper::*;
        let oper = binary.oper.clone();

        let left = self.visit_expr(&binary.left)?;
        let right = self.visit_expr(&binary.right)?;

        let left = left
            .as_value()
            .ok_or_else(|| RuntimeError::MismatchedType)?;
        let right = right
            .as_value()
            .ok_or_else(|| RuntimeError::MismatchedType)?;

        // TODO: error if failed to cast
        Ok(match oper {
            Equal | NotEqual => {
                let cp = logic::obj_eq(left, right).ok_or_else(|| RuntimeError::MismatchedType)?;
                LoxObj::bool(cp)
            }

            Less | LessEqual | Greater | GreaterEqual => {
                let ord =
                    logic::obj_cmp(left, right).ok_or_else(|| RuntimeError::MismatchedType)?;
                // TODO: no branch
                LoxObj::bool(match binary.oper {
                    Less => ord == Ordering::Less,
                    LessEqual => ord != Ordering::Greater,
                    Greater => ord == Ordering::Greater,
                    GreaterEqual => ord != Ordering::Less,
                    _ => panic!(),
                })
            }

            Minus | Plus | Div | Mul => match oper {
                Minus => logic::obj_minus(left, right),
                Plus => logic::obj_plus(left, right),
                Div => logic::obj_div(left, right),
                Mul => logic::obj_mul(left, right),
                _ => panic!(),
            }
            .ok_or_else(|| RuntimeError::MismatchedType)?,
        })
    }

    /// `&&`, `||`
    fn visit_logic_expr(&mut self, logic: &LogicArgs) -> Result<LoxObj> {
        let oper = logic.oper.clone();
        let left_truthy = self.visit_expr(&logic.left)?.is_truthy();
        Ok(match oper {
            LogicOper::Or => {
                if left_truthy {
                    LoxObj::bool(true)
                } else {
                    LoxObj::bool(self.visit_expr(&logic.right)?.is_truthy())
                }
            }
            LogicOper::And => {
                LoxObj::bool(left_truthy && self.visit_expr(&logic.right)?.is_truthy())
            }
        })
    }

    fn visit_var_expr(&mut self, name: &str) -> Result<LoxObj> {
        self.env.borrow().get(name)
    }

    fn visit_assign_expr(&mut self, assign: &AssignArgs) -> Result<LoxObj> {
        let obj = self.eval_expr(&assign.expr)?;
        self.env
            .borrow_mut()
            .assign(assign.name.as_str(), obj.clone())?;
        Ok(obj)
    }

    fn visit_call_expr(&mut self, call: &CallArgs) -> Result<LoxObj> {
        if let LoxObj::Callable(ref fn_obj) = self.eval_expr(&call.callee)? {
            let obj = self.invoke(fn_obj, &call.args)?.unwrap_or_else(|| LoxObj::nil());
            Ok(obj)
        } else {
            Err(RuntimeError::MismatchedType)
        }
    }
}
