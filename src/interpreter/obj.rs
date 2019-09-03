use crate::abs::expr::*;

/// The primary object at runtime interpreting
///
/// primary → "true" | "false" | "nil"
///         | NUMBER | STRING
///         | "(" expression ")"
///         | IDENTIFIER ;
#[derive(Clone, Debug, PartialEq)]
pub enum LoxObj {
    // TODO: literal -> value
    Value(LiteralArgs),
    // Literal(LiteralArgs),
    // Variable(String),
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
