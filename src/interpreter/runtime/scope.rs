use crate::interpreter::runtime::object::LoxObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DEFAULT_SCOPE_SIZE: usize = 32;

#[derive(Debug)]
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

    pub fn size(&self) -> usize {
        self.map.len()
    }

    pub fn get(&self, name: &str) -> Option<LoxObject> {
        if let Some(v) = self.map.get(name) {
            return Some(v.clone());
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.borrow().get(name);
        }
        None
    }

    // this writes directly into our map without even checking the upper scope.
    pub fn set_local(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
        self.map.insert(name.to_string(), value)
    }

    // this will set the variable and let you know if it was ultimately set or not.
    pub fn set(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
        if self.map.contains_key(name) {
            return self.map.insert(name.to_string(), value);
        }
        if let Some(parent) = self.parent.as_ref() {
            return parent.borrow_mut().set(name, value);
        }
        None
    }

    // creates a scope pointing at the same parent.
    pub fn sibling(&self) -> Option<Scope> {
        self.parent.as_ref().map(|p| p.clone().into())
    }

    pub fn has_local(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn parent(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.as_ref().map(|p| p.clone())
    }

    pub fn with_parent(mut self, parent: Rc<RefCell<Scope>>) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn delete(&mut self, key: &str) {
        self.map.remove(key);
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
