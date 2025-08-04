use crate::interpreter::runtime::object::LoxObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,

    // Mapping from name → slot index in `values`
    slots: HashMap<String, usize>, // Flat storage of this frame’s locals
    values: Vec<LoxObject>,
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            parent,
            slots: HashMap::new(),
            values: Vec::new(),
        }
    }

    /// Declare a slot for `name`, returning its index.
    pub fn declare(&mut self, name: &str) -> usize {
        let idx = self.values.len();
        self.values.push(LoxObject::new_nil());
        self.slots.insert(name.to_string(), idx);
        idx
    }

    /// Define (or redefine) the value in an existing slot
    pub fn define(&mut self, name: &str, value: LoxObject) {
        if let Some(&idx) = self.slots.get(name) {
            self.values[idx] = value;
        }
    }

    /// Walk up `distance` scopes and return the slot’s value.
    pub fn get_at(&self, distance: usize, slot: usize) -> LoxObject {
        if distance == 0 {
            // should be good to go as long as everything was declared correctly.
            return self.values[slot].clone();
        }
        self.parent
            .as_ref()
            .unwrap()
            .borrow()
            .get_at(distance - 1, slot)
    }

    /// Same, but mutate.
    pub fn set_at(&mut self, distance: usize, slot: usize, value: LoxObject) {
        if distance == 0 {
            self.values[slot] = value;
        } else {
            self.parent
                .as_ref()
                .unwrap()
                .borrow_mut()
                .set_at(distance - 1, slot, value);
        }
    }

    pub fn parent(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.clone()
    }

    pub fn print(&self) {
        self.print_impl("");
    }

    fn print_impl(&self, prefix: &str) {
        println!("{}slots -> {:?}", prefix, self.slots);
        println!("{}values -> {:?}", prefix, self.values);
        if let Some(ref p) = self.parent() {
            p.borrow().print_impl(format!("{}  ", prefix).as_str());
        }
    }
}

// #[derive(Debug)]
// pub struct Scope {
//     parent: Option<Rc<RefCell<Scope>>>,
//     map: HashMap<String, LoxObject>,
// }

// impl Scope {
//     pub fn new(parent: Option<Rc<RefCell<Scope>>>, map_size: usize) -> Self {
//         Self {
//             parent,
//             map: HashMap::with_capacity(map_size),
//         }
//     }

//     pub fn size(&self) -> usize {
//         self.map.len()
//     }

//     pub fn get(&self, name: &str) -> Option<LoxObject> {
//         if let Some(v) = self.map.get(name) {
//             return Some(v.clone());
//         }
//         if let Some(parent) = self.parent.as_ref() {
//             return parent.borrow().get(name);
//         }
//         None
//     }

//     // this writes directly into our map without even checking the upper scope.
//     pub fn set_local(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
//         self.map.insert(name.to_string(), value)
//     }

//     // this will set the variable and let you know if it was ultimately set or not.
//     pub fn set(&mut self, name: &str, value: LoxObject) -> Option<LoxObject> {
//         if self.map.contains_key(name) {
//             return self.map.insert(name.to_string(), value);
//         }
//         if let Some(parent) = self.parent.as_ref() {
//             return parent.borrow_mut().set(name, value);
//         }
//         None
//     }

//     // creates a scope pointing at the same parent.
//     pub fn sibling(&self) -> Option<Scope> {
//         self.parent.as_ref().map(|p| p.clone().into())
//     }

//     pub fn has_local(&self, key: &str) -> bool {
//         self.map.contains_key(key)
//     }

//     pub fn parent(&self) -> Option<Rc<RefCell<Scope>>> {
//         self.parent.as_ref().map(|p| p.clone())
//     }

//     pub fn with_parent(mut self, parent: Rc<RefCell<Scope>>) -> Self {
//         self.parent = Some(parent);
//         self
//     }

//     pub fn delete(&mut self, key: &str) {
//         self.map.remove(key);
//     }
// }

impl Default for Scope {
    fn default() -> Self {
        Self::new(None)
    }
}

impl From<Rc<RefCell<Scope>>> for Scope {
    fn from(value: Rc<RefCell<Scope>>) -> Self {
        Self::new(Some(value))
    }
}
