use crate::ast::expr::*;

/// Anything at runtime
///
/// primary → "true" | "false" | "nil"
///         | NUMBER | STRING
///         | "(" expression ")"
///         | IDENTIFIER ;
#[derive(Clone, Debug, PartialEq)]
pub enum LoxObj {
    Value(LoxValue),
    Callable(LoxFn),
    Variable(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoxValue {
    Nil,
    Bool(bool),
    StringLit(String),
    Number(f64),
}

impl LoxValue {
    pub fn from_lit(lit: &LiteralArgs) -> Self {
        match lit {
            LiteralArgs::Nil => LoxValue::Nil,
            LiteralArgs::Bool(b) => LoxValue::Bool(b.clone()),
            LiteralArgs::StringLit(s) => LoxValue::StringLit(s.clone()),
            LiteralArgs::Number(n) => LoxValue::Number(n.clone()),
        }
    }
}

impl From<LoxValue> for LoxObj {
    fn from(value: LoxValue) -> Self {
        LoxObj::Value(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoxFn {}

impl LoxFn {
    pub fn arity(&self) -> usize {
        0
    }
}

// pub struct LoxSignature {}

// pub enum ValueArgs {}

impl LoxObj {
    pub fn bool(b: bool) -> Self {
        LoxObj::Value(LoxValue::Bool(b))
    }

    pub fn from_lit(lit: &LiteralArgs) -> Self {
        LoxObj::Value(LoxValue::from_lit(lit))
    }

    pub fn is_truthy(&self) -> bool {
        use LoxValue::*;
        let value = match self {
            LoxObj::Value(lit) => lit,
            _ => return false,
        };
        match value {
            Nil | Bool(true) => true,
            _ => false,
        }
    }

    pub fn as_value(&self) -> Option<&LoxValue> {
        match self {
            LoxObj::Value(ref value) => Some(value),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            LoxObj::Value(LoxValue::Number(n)) => Some(n.clone()),
            _ => None,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LoxObj::Value(LoxValue::Nil) => true,
            _ => false,
        }
    }
}
