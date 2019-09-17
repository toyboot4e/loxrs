use ::std::cell::RefCell;
use ::std::rc::Rc;

use super::env::Env;
use super::visitor::StmtVisitor;

use crate::ast::{expr::*, stmt::*};
use crate::runtime::obj::{LoxObj, LoxValue};
use crate::ast::PrettyPrint;

/// Runtime error when evaluating expressions.
#[derive(Debug)]
pub enum RuntimeError {
    // TODO: use more detailed context
    MismatchedType,
    /// Tried to lookup undefined variable
    Undefined(String),
    // TODO: enable overwriting
    DuplicateDeclaration(String),
}

type Result<T> = ::std::result::Result<T, RuntimeError>;

pub struct Interpreter {
    env: Rc<RefCell<Env>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Env::new())),
        }
    }

    /// Implemented with Visitor pattern
    pub fn interpret(&mut self, stmt: &Stmt) -> Result<()> {
        self.visit_stmt(stmt)
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

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        let v = self.eval_expr(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, print: &PrintArgs) -> Result<()> {
        let obj = self.eval_expr(&print.expr)?;
        // println!("{}", expr.pretty_print());
        println!("{}", obj.pretty_print());
        Ok(())
    }

    fn visit_var_dec_stmt(&mut self, var: &VarDecArgs) -> Result<()> {
        let name = &var.name;
        let obj = self.eval_expr(&var.init)?;
        self.env.borrow_mut().define(name, obj)?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, if_: &IfArgs) -> Result<()> {
        if self.eval_expr(&if_.condition)?.is_truthy() {
            self.interpret(&if_.if_true)
        } else if let Some(if_false) = if_.if_false.as_ref() {
            self.interpret(if_false)
        } else {
            Ok(())
        }
    }

    fn visit_block_stmt(&mut self, block: &BlockArgs) -> Result<()> {
        let prev = Rc::clone(&self.env);
        self.env = Rc::new(RefCell::new(Env::from_parent(&prev)));
        if let Some(err_result) = block
            .stmts
            .iter()
            .map(|x| self.interpret(x))
            .find(|x| x.is_err())
        {
            self.env = prev;
            err_result
        } else {
            self.env = prev;
            Ok(())
        }
    }

    fn visit_while_stmt(&mut self, while_: &WhileArgs) -> Result<()> {
        while self.eval_expr(&while_.condition)?.is_truthy() {
            self.visit_block_stmt(&while_.block)?;
        }
        Ok(())
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
use ::std::cmp::Ordering;

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
        let env = self.env.borrow_mut();
        match env.get(name) {
            Ok(obj) => Ok(obj),
            Err(_) => Err(RuntimeError::Undefined(name.to_string())),
        }
    }

    fn visit_assign_expr(&mut self, assign: &AssignArgs) -> Result<LoxObj> {
        let obj = self.eval_expr(&assign.expr)?;
        self.env
            .borrow_mut()
            .assign(assign.name.as_str(), obj.clone())?;
        Ok(obj)
    }

    fn visit_call_expr(&mut self, call: &CallArgs) -> Result<LoxObj> {
        if let LoxObj::Callable(ref f) = self.eval_expr(&call.callee)? {
            // call
            unimplemented!()
        } else {
            Err(RuntimeError::MismatchedType)
        }
    }
}
