use ::std::cell::RefCell;
use ::std::rc::Rc;

use super::env::Env;
use super::visitor::StmtVisitor;

use crate::abs::expr::*;
use crate::abs::stmt::*;
use crate::runtime::obj::LoxObj;

/// Runtime error when evaluating expressions.
#[derive(Debug)]
pub enum RuntimeError {
    MismatchedType,
    /// Tried to lookup undefined variable
    Undefined(String),
    // TODO: enable overwriting
    DuplicateDefinition(String),
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
        use LiteralArgs::*;
        match lit {
            Nil => "<nil>".to_string(),
            Bool(b) => b.to_string(),
            // TODO: avoid cloning?
            StringL(s) => s.clone(),
            Number(n) => n.to_string(),
        }
    } else {
        "ERROR WHEN STRINGIFY LOX OBJECT".to_string()
    }
}

// interprete functions
impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_expr_stmt(&mut self, expr: &Expr) -> Result<()> {
        let v = self.eval_expr(expr)?;
        println!("expr: {:?}", v);
        Ok(())
    }

    fn visit_print_stmt(&mut self, print: &PrintArgs) -> Result<()> {
        let v = self.eval_expr(&print.expr)?;
        println!("print: {:?}", stringify_obj(&v));
        Ok(())
    }

    fn visit_var_dec_stmt(&mut self, var: &VarDecArgs) -> Result<()> {
        let name = &var.name;
        let obj = self.eval_expr(&var.init)?;
        println!("var_dec: {:?} = {:?}", name, &obj);
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

    fn visit_block_stmt(&mut self, block: &[Stmt]) -> Result<()> {
        let prev = Rc::clone(&self.env);
        self.env = Rc::new(RefCell::new(Env::from_parent(&prev)));
        if let Some(err_result) = block.iter().map(|x| self.interpret(x)).find(|x| x.is_err()) {
            self.env = prev;
            err_result
        } else {
            self.env = prev;
            Ok(())
        }
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
use LiteralArgs as Lit;
use LoxObj::Value as ValObj;

mod logic {
    //! Operator overloading for specific LoxObj_s.

    use crate::abs::expr::*;
    use crate::runtime::obj::LoxObj;
    use std::cmp::Ordering;
    use LiteralArgs as Lit;

    pub fn obj_eq(left: &LiteralArgs, right: &LiteralArgs) -> Option<bool> {
        Some(match (left, right) {
            (Lit::Number(n1), Lit::Number(n2)) => n1 == n2,
            (Lit::Bool(b1), Lit::Bool(b2)) => b1 == b2,
            (Lit::StringL(s1), Lit::StringL(s2)) => s1 == s2,
            _ => return None,
        })
    }

    pub fn obj_cmp(left: &LiteralArgs, right: &LiteralArgs) -> Option<Ordering> {
        match (left, right) {
            (Lit::Number(n1), Lit::Number(n2)) => n1.partial_cmp(n2),
            _ => None,
        }
    }

    pub fn obj_plus(left: &LiteralArgs, right: &LiteralArgs) -> Option<LoxObj> {
        use LiteralArgs::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 + n2),
            (StringL(s1), StringL(s2)) => StringL(format!("{}{}", s1, s2)),
            _ => return None,
        }))
    }

    pub fn obj_minus(left: &LiteralArgs, right: &LiteralArgs) -> Option<LoxObj> {
        use LiteralArgs::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 - n2),
            _ => return None,
        }))
    }

    pub fn obj_div(left: &LiteralArgs, right: &LiteralArgs) -> Option<LoxObj> {
        use LiteralArgs::*;
        Some(LoxObj::Value(match (left, right) {
            (Number(n1), Number(n2)) => Number(n1 / n2),
            _ => return None,
        }))
    }

    pub fn obj_mul(left: &LiteralArgs, right: &LiteralArgs) -> Option<LoxObj> {
        use LiteralArgs::*;
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
    fn visit_literal_expr(&mut self, literal: &LiteralArgs) -> Result<LoxObj> {
        Ok(ValObj(literal.clone()))
    }

    fn visit_unary_expr(&mut self, unary: &UnaryArgs) -> Result<LoxObj> {
        let obj = self.visit_expr(&unary.expr)?;
        use UnaryOper::*;
        match &unary.oper {
            Minus => {
                let n = obj.as_num().ok_or_else(|| RuntimeError::MismatchedType)?;
                Ok(LoxObj::Value(Lit::Number(-n)))
            }
            Not => Ok(LoxObj::bool(!obj.is_truthy())),
        }
    }

    fn visit_binary_expr(&mut self, binary: &BinaryArgs) -> Result<LoxObj> {
        use BinaryOper::*;
        let oper = binary.oper.clone();

        let left = self.visit_expr(&binary.left)?;
        let right = self.visit_expr(&binary.right)?;

        let left = left.as_lit().ok_or_else(|| RuntimeError::MismatchedType)?;
        let right = right.as_lit().ok_or_else(|| RuntimeError::MismatchedType)?;

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

    fn visit_logic_expr(&mut self, unary: &LogicArgs) -> Result<LoxObj> {
        let oper = unary.oper.clone();
        let left_truthy = self.visit_expr(&unary.left)?.is_truthy();
        if left_truthy && oper == LogicOper::Or {
            return Ok(LoxObj::bool(true));
        }
        let right_truthy = self.visit_expr(&unary.right)?.is_truthy();
        Ok(LoxObj::bool(right_truthy))
    }

    fn visit_var_expr(&mut self, name: &str) -> Result<LoxObj> {
        let env = self.env.borrow_mut();
        match env.get(name) {
            Ok(obj) => Ok(obj.clone()), // FIXME
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
}
