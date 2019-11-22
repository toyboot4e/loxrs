//! Runtime representations of objects, separated from AST

use crate::ast::{
    expr::*,
    pretty_printer::{self, PrettyPrint},
    stmt::{ClassDeclArgs, FnDeclArgs, Params, Stmt},
};
use crate::runtime::{env::Env, Result, RuntimeError};
use ::std::cell::RefCell;
use ::std::collections::HashMap;
use ::std::fmt::Write;
use ::std::rc::Rc;

/// Runtime object which represents anything
#[derive(Clone, Debug)]
pub enum LoxObj {
    Value(LoxValue),
    Callable(LoxFn),
    // TODO: consider using Rc or not (to reference from instance)
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl LoxObj {
    pub fn nil() -> Self {
        LoxObj::Value(LoxValue::Nil)
    }

    pub fn f(def: &FnDeclArgs, closure: &Rc<RefCell<Env>>) -> Self {
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

// TODO: remove native functions
/// Runtime function object (expect class names as constructors)
///
/// It's not so expensive to copy a `LoxFn`
#[derive(Clone, Debug)]
pub enum LoxFn {
    /// User defined function
    User(LoxUserFn),
    /// A native function embedded in rulox
    Clock,
    // /// Generic native function identifier
    // Native(String, Option<Args>),
}

impl LoxFn {
    pub fn from_decl(decl: &FnDeclArgs, closure: &Rc<RefCell<Env>>) -> Self {
        LoxFn::User(LoxUserFn::from_def(decl, closure))
    }

    pub fn bind(&self, instance: &Rc<RefCell<LoxInstance>>) -> Result<Self> {
        match self {
            LoxFn::User(f) => Ok(LoxFn::User(f.bind(instance)?)),
            _ => Err(RuntimeError::CantBind),
        }
    }
}

/// Runtime representaiton of a user-defined function.
#[derive(Clone, Debug)]
pub struct LoxUserFn {
    /// `Rc` for slicing the body to instance methods
    pub body: Rc<Vec<Stmt>>,
    pub params: Params,
    // TODO: disable mutation
    pub closure: Rc<RefCell<Env>>,
    // TODO: should have name in field or not
}

impl LoxUserFn {
    pub fn from_def(decl: &FnDeclArgs, closure: &Rc<RefCell<Env>>) -> Self {
        Self {
            body: Rc::clone(&decl.body),
            params: decl.params.clone(),
            closure: Rc::clone(closure),
        }
    }

    pub fn bind(&self, instance: &Rc<RefCell<LoxInstance>>) -> Result<LoxUserFn> {
        let mut env = Env::from_parent(&self.closure);
        env.define("@", LoxObj::Instance(Rc::clone(instance)))?;
        Ok(LoxUserFn {
            body: Rc::clone(&self.body),
            params: self.params.clone(),
            closure: Rc::new(RefCell::new(env)),
        })
    }
}

/// Runtime representation of a class
#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxFn>,
}

impl LoxClass {
    pub fn from_decl(decl: &ClassDeclArgs, closure: &Rc<RefCell<Env>>) -> Self {
        Self {
            name: decl.name.clone(),
            methods: decl
                .methods
                .iter()
                .map(|m| (m.name.to_owned(), LoxFn::from_decl(m, closure)))
                .collect(),
        }
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFn> {
        self.methods.get(name).map(|m| m.clone())
    }
}

/// Runtime representation of an instance of a `LoxClass`
#[derive(Clone, Debug)]
pub struct LoxInstance {
    // FIXME: use indirect access to a class
    pub class: Rc<LoxClass>,
    fields: HashMap<String, LoxObj>,
}

#[derive(Clone, Debug)]
pub struct AssignHandle {
    did_reassign: bool,
}

impl LoxInstance {
    pub fn new(class: &Rc<LoxClass>) -> Self {
        Self {
            class: Rc::clone(class),
            // TODO: initialize with fields
            fields: HashMap::new(),
        }
    }

    /// Borrows self
    pub fn get(self_: &Rc<RefCell<LoxInstance>>, name: &str) -> Result<LoxObj> {
        // variable > method
        if let Some(obj) = self_.borrow().fields.get(name) {
            Ok(obj.clone())
        } else if let Some(method) = self_.borrow().class.find_method(name) {
            let binded = method.bind(self_)?;
            Ok(LoxObj::Callable(binded))
        } else {
            Err(RuntimeError::NoFieldWithName(name.to_string()))
        }
    }

    pub fn set(&mut self, name: &str, value: LoxObj) {
        self.fields.insert(name.to_owned(), value);
    }

    pub fn try_assign(&mut self, name: &str, value: LoxObj) -> Result<AssignHandle> {
        if let Some(obj) = self.fields.get_mut(name) {
            Err(RuntimeError::ReassignDisabled)
        } else {
            // FIXME: reduce cloning
            Ok(AssignHandle {
                did_reassign: self.fields.insert(name.to_owned(), value).is_some(),
            })
        }
    }

    pub fn try_reassign(&mut self, name: &str, value: LoxObj) -> Result<()> {
        if let Some(obj) = self.fields.get_mut(name) {
            *obj = value;
            Ok(())
        } else {
            Err(RuntimeError::NoFieldWithName(name.to_owned()))
        }
    }
}

// impl PrettyPrint for the `print` native function

impl PrettyPrint for LoxValue {
    fn pretty_print(&self) -> String {
        match *self {
            LoxValue::Nil => "Nil".into(),
            LoxValue::Bool(b) => {
                if b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            LoxValue::StringLit(ref s) => format!("\"{}\"", s.clone()),
            LoxValue::Number(n) => n.to_string(),
        }
    }
}

impl PrettyPrint for LoxObj {
    fn pretty_print(&self) -> String {
        match self {
            LoxObj::Value(value) => value.pretty_print(),
            LoxObj::Callable(call) => call.pretty_print(),
            LoxObj::Class(class) => class.pretty_print(),
            // TODO: test if it will get panic
            LoxObj::Instance(instance) => instance.borrow().pretty_print(),
        }
    }
}

impl PrettyPrint for LoxFn {
    fn pretty_print(&self) -> String {
        match self {
            LoxFn::Clock => "(fn clock)".into(),
            LoxFn::User(ref user) => user.pretty_print(),
        }
    }
}

impl PrettyPrint for LoxUserFn {
    fn pretty_print(&self) -> String {
        let mut s = String::new();
        write!(s, "(fn ").unwrap();
        pretty_printer::write_slice(&mut s, &self.params);
        write!(s, "\n").unwrap();
        pretty_printer::write_indent(&mut s, 1);
        pretty_printer::write_stmts(&mut s, 1, &self.body);
        write!(s, ")").unwrap();
        s
    }
}

// TODO: use & make writing methods
impl PrettyPrint for LoxClass {
    fn pretty_print(&self) -> String {
        format!("(class {})", &self.name)
    }
}

impl PrettyPrint for LoxInstance {
    fn pretty_print(&self) -> String {
        let mut s = String::new();
        self::write_instance(&mut s, self);
        s
    }
}

fn write_instance(s: &mut String, instance: &LoxInstance) {
    write!(s, "(instance ").unwrap();
    self::write_class_obj(s, &instance.class);
    write!(s, " (").unwrap();
    for (name, method) in instance.fields.iter() {
        write!(s, "({} ", name).unwrap();
        write!(s, ")").unwrap();
    }
    write!(s, ")").unwrap();
}

fn write_class_obj(s: &mut String, class: &LoxClass) {
    write!(s, "(class {})", &class.name).unwrap();
}
