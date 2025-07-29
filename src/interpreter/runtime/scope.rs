use crate::interpreter::runtime::value::LoxObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DEFAULT_SCOPE_SIZE: usize = 32;

pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    map: HashMap<String, LoxObject>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>, map_size: usize) -> Self {
        Self {
            parent,
            map: HashMap::with_capacity(map_size),
        }
    }

    pub fn resolve(&self, name: &str) -> Option<LoxObject> {
        if let Some(v) = self.map.get(name) {
            return Some(v.clone());
        }

        if let Some(ref parent) = self.parent {
            return parent.borrow().resolve(name);
        }

        None
    }

    pub fn declare(&mut self, name: &str, value: LoxObject) {
        self.map.insert(name.to_string(), value);
    }

    // creates a scope pointing at the same parent.
    pub fn sibling(&self) -> Option<Scope> {
        self.parent.as_ref().map(|p| p.clone().into())
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new(None, DEFAULT_SCOPE_SIZE)
    }
}

impl From<Rc<RefCell<Scope>>> for Scope {
    fn from(value: Rc<RefCell<Scope>>) -> Self {
        Self::new(Some(value), 32)
    }
}
