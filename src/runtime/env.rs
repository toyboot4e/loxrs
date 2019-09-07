use crate::runtime::interpreter::RuntimeError;
use crate::runtime::obj::LoxObj;
use ::std::cell::RefCell;
use ::std::collections::HashMap;
use ::std::rc::{Rc, Weak};

type Result<T> = ::std::result::Result<T, RuntimeError>;

pub struct Env {
    map: RefCell<HashMap<String, LoxObj>>,
    /// Enclosing environment (if any)
    parent: Weak<RefCell<Self>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            map: RefCell::new(HashMap::new()),
            parent: Weak::new(),
        }
    }

    pub fn from_parent(parent: &Rc<RefCell<Self>>) -> Self {
        Env {
            map: RefCell::new(HashMap::new()),
            parent: Rc::downgrade(parent),
        }
    }

    // TODO: check non-recursive solution in CLox
    // TODO: `get` without cloning
    /// Looks up enclosing environment and clones the found object
    pub fn get(&self, name: &str) -> Result<LoxObj> {
        match self.map.borrow().get(name) {
            Some(obj) => Ok(obj.clone()),
            None => match self.parent.upgrade() {
                Some(parent) => parent.borrow().get(name),
                None => Err(RuntimeError::Undefined(name.to_string())),
            },
        }
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
        if map.contains_key(name) {
            println!("assingn {:?}", &obj);
            map.insert(name.to_owned(), obj);
            Ok(())
        } else {
            match self.parent.upgrade() {
                Some(rc) => rc.borrow_mut().assign(name, obj),
                None => Err(RuntimeError::Undefined(name.to_string())),
            }
        }
    }
}
