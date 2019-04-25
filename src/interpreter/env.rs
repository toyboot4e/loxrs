use crate::interpreter::obj::LoxObj;
use std::collections::HashMap;

pub enum EnvironmentError {
    None,
}

pub struct Environment {
    map: HashMap<String, LoxObj>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &str) -> Result<&LoxObj, EnvironmentError> {
        self.map.get(name).ok_or_else(|| EnvironmentError::None)
    }

    pub fn define(&mut self, name: &str, obj: LoxObj) {
        self.map.insert(name.into(), obj);
    }
}
