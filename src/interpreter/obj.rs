use crate::abs::expr::*;
use std::cmp;

#[derive(Clone, Debug)]
pub enum LoxObj {
    // TODO: literal -> value
    Value(LiteralArgs),
}

// pub enum ValueArgs {}

impl From<LiteralArgs> for LoxObj {
    fn from(item: LiteralArgs) -> LoxObj {
        LoxObj::Value(item)
    }
}

impl LoxObj {
    pub fn bool(b: bool) -> Self {
        LoxObj::Value(LiteralArgs::Bool(b))
    }

    pub fn is_truthy(&self) -> bool {
        use LiteralArgs::*;
        let lit = match self {
            LoxObj::Value(lit) => lit,
            _ => return false,
        };
        match lit {
            Nil | Bool(true) => true,
            _ => false,
        }
    }

    pub fn as_lit(&self) -> Option<&LiteralArgs> {
        match self {
            LoxObj::Value(ref args) => Some(args),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            LoxObj::Value(LiteralArgs::Number(n)) => Some(n.clone()),
            _ => None,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LoxObj::Value(LiteralArgs::Nil) => true,
            _ => false,
        }
    }
}

use LoxObj::Value as ValObj;

impl cmp::PartialEq for LoxObj {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&ValObj(ref lhs), &ValObj(ref rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl cmp::PartialOrd for LoxObj {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (ValObj(ref left), ValObj(ref right)) => left.partial_cmp(right),
            _ => None,
        }
    }
}

// impl fmt::Display for LoxObj {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         use LoxObj::*;
//         match *self {
//             Literal(ref value) => value.fmt(),
//         }
//     }
// }
