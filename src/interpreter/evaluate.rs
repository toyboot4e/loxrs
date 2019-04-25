use crate::abs::expr::*;
use crate::interpreter::{
    interpreter::{Interpreter, RuntimeError},
    obj::LoxObj,
    visitor::ExprVisitor,
};
use std::cmp::Ordering;

type Result<T> = ::std::result::Result<T, RuntimeError>;
// TODO: using Value struct
use LiteralArgs as Lit;
use LoxObj::Value as ValObj;

mod logic {
    use crate::abs::expr::*;
    use crate::interpreter::obj::LoxObj;
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

impl ExprVisitor<Result<LoxObj>> for Interpreter {
    fn visit_literal(&mut self, literal: &LiteralArgs) -> Result<LoxObj> {
        Ok(ValObj(literal.clone()))
    }

    fn visit_unary(&mut self, unary: &UnaryArgs) -> Result<LoxObj> {
        let obj = self.visit_expr(&unary.expr)?;
        use UnaryOper::*;
        match &unary.oper {
            Minus => {
                let n = obj.as_num().ok_or_else(|| RuntimeError::MismatchedType)?;
                Ok(LoxObj::Value(Lit::Number(-n)))
            }
            Not => Ok(LoxObj::bool(obj.is_truthy())),
        }
    }

    fn visit_binary(&mut self, binary: &BinaryArgs) -> Result<LoxObj> {
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

    fn visit_logic(&mut self, unary: &LogicArgs) -> Result<LoxObj> {
        let oper = unary.oper.clone();
        let left_truthy = self.visit_expr(&unary.left)?.is_truthy();
        if left_truthy && oper == LogicOper::Or {
            return Ok(LoxObj::bool(true));
        }
        let right_truthy = self.visit_expr(&unary.right)?.is_truthy();
        Ok(LoxObj::bool(right_truthy))
    }
}
