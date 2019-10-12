use crate::runtime::{obj::LoxObj, RuntimeError};
use ::std::cell::RefCell;
use ::std::collections::HashMap;
use ::std::rc::{Rc, Weak};

type Result<T> = ::std::result::Result<T, RuntimeError>;

#[derive(Clone, Debug)]
pub struct Env {
    /// Objects; variables or functions
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

    // TODO: check non-recursive solution in CLox and compare with it
    // TODO: `get` without cloning?
    /// Looks up in this or enclosing environment dynamically and clones the object found
    pub fn get(&self, name: &str) -> Result<LoxObj> {
        match self.map.borrow().get(name) {
            Some(obj) => Ok(obj.clone()),
            None => match self.parent.upgrade() {
                Some(parent) => parent.borrow().get(name),
                None => Err(RuntimeError::Undefined(name.to_string())),
            },
        }
    }

    /// Looks up *this* environment, doesn't looking into enclosing ones
    pub fn contains(&self, name: &str) -> bool {
        self.map.borrow().get(name).is_some()
    }

    pub fn define(&mut self, name: &str, obj: LoxObj) -> Result<()> {
        if self.map.borrow().contains_key(name) {
            // we disable overwriting a previous variable with same name
            Err(RuntimeError::DuplicateDeclaration(name.to_string()))
        } else {
            self.map.borrow_mut().insert(name.to_owned(), obj);
            Ok(())
        }
    }

    pub fn assign(&mut self, name: &str, obj: LoxObj) -> Result<()> {
        let mut map = self.map.borrow_mut();
        if map.contains_key(name) {
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

/// Efficient methods trusting Resolver's work
impl Env {
    /// Looks up an enclosing environment in a distance, trusting the length.
    /// Panics if it reaches unexisting environment.
    fn ancestor(&self, d: usize) -> Rc<RefCell<Env>> {
        let ancestor = (0..d)
            .scan(self.parent.upgrade().unwrap(), |env, _| {
                Some(env.borrow().parent.upgrade().unwrap())
            })
            .last()
            .unwrap();
        ancestor.clone()
    }

    pub fn get_resolved(&self, name: &str, d: usize) -> Result<LoxObj> {
        // FIXME: may panic
        match self.ancestor(d).borrow().map.borrow().get(name) {
            Some(name) => Ok(name.clone()),
            _ => Err(RuntimeError::Undefined(name.to_string())),
        }
    }

    pub fn assign_resolved(&mut self, name: &str, obj: LoxObj, d: usize) -> Result<()> {
        self.ancestor(d).borrow_mut().assign(name, obj)
    }
}
