use crate::interpreter::interpreter::RuntimeError;
use crate::interpreter::obj::LoxObj;
use ::std::cell::RefCell;
use ::std::collections::HashMap;
use ::std::rc::{Rc, Weak};

type Result<T> = ::std::result::Result<T, RuntimeError>;

pub struct Env {
    map: RefCell<HashMap<String, LoxObj>>,
    /// Enclosing environment (if any)
    parent: Weak<Self>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            map: RefCell::new(HashMap::new()),
            parent: Weak::new(),
        }
    }

    pub fn from_parent(parent: &Rc<Self>) -> Self {
        Env {
            map: RefCell::new(HashMap::new()),
            parent: Rc::downgrade(parent),
        }
    }

    /// Clones the `LoxObj` with the `name`
    pub fn get(&self, name: &str) -> Result<LoxObj> {
        self.map
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::Undefined(name.to_string()))
    }

    pub fn define(&mut self, name: &str, obj: LoxObj) -> Result<()> {
        if self.map.borrow().contains_key(name) {
            Err(RuntimeError::DuplicateDefinition(name.to_string()))
        } else {
            self.map.borrow_mut().insert(name.to_owned(), obj);
            Ok(())
        }
    }

    pub fn assign(&mut self, name: &str, obj: LoxObj) -> Result<()> {
        let mut map = self.map.borrow_mut();
        match map.contains_key(name) {
            false => {
                map.insert(name.to_owned(), obj);
                Ok(())
            }
            true => Err(RuntimeError::Undefined(name.to_string())),
        }
    }
}
