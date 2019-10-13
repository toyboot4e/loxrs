//! Object (value, variable or function) definitions

use crate::ast::expr::*;
use crate::ast::stmt::{FnDef, Params, Stmt};
use crate::runtime::env::Env;
use ::std::cell::RefCell;
use ::std::rc::Rc;

/// Runtime object which represents anything
#[derive(Clone, Debug)]
pub enum LoxObj {
    Value(LoxValue),
    Callable(LoxFn),
}

impl LoxObj {
    pub fn nil() -> Self {
        LoxObj::Value(LoxValue::Nil)
    }

    pub fn f(def: &FnDef, closure: &Rc<RefCell<Env>>) -> Self {
        LoxObj::Callable(LoxFn::User(LoxUserFn::from_def(def, closure)))
    }
}

/// Runtime value
// TODO: use traits and share instances between `LoxObj` & `LiteralData`
#[derive(Clone, Debug, PartialEq)]
pub enum LoxValue {
    Nil,
    Bool(bool),
    StringLit(String),
    Number(f64),
}

impl LoxValue {
    pub fn from_lit(lit: &LiteralData) -> Self {
        match lit {
            LiteralData::Nil => LoxValue::Nil,
            LiteralData::Bool(b) => LoxValue::Bool(b.clone()),
            LiteralData::StringLit(s) => LoxValue::StringLit(s.clone()),
            LiteralData::Number(n) => LoxValue::Number(n.clone()),
        }
    }
}

impl From<LoxValue> for LoxObj {
    fn from(value: LoxValue) -> Self {
        LoxObj::Value(value)
    }
}

impl LoxObj {
    pub fn bool(b: bool) -> Self {
        LoxObj::Value(LoxValue::Bool(b))
    }

    pub fn from_lit(lit: &LiteralData) -> Self {
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

/// Runtime function object
#[derive(Clone, Debug)]
pub enum LoxFn {
    /// User defined function
    User(LoxUserFn),
    /// A native function embedded in rulox
    Clock,
    // /// Generic native function identifier
    // Native(String, Option<Args>),
}

/// Runtime user-defined function
#[derive(Clone, Debug)]
pub struct LoxUserFn {
    pub body: Vec<Stmt>,
    pub params: Option<Params>,
    // TODO: disable mutation
    pub closure: Rc<RefCell<Env>>,
}

impl LoxUserFn {
    pub fn from_def(def: &FnDef, closure: &Rc<RefCell<Env>>) -> Self {
        let env = Env::from_parent(&closure);
        Self {
            body: def.body.stmts.clone(),
            params: def.params.clone(),
            closure: Rc::clone(closure),
        }
    }
}
