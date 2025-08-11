use super::function::Function;
use super::object::LoxObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

const DEFAULT_PROPERTY_HASH_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    methods: HashMap<String, Rc<Function>>,
    statics: HashMap<String, Rc<Function>>,
    super_class: Option<Rc<Class>>,
    init: Option<Rc<Function>>,
}

impl Class {
    pub fn new(
        name: String,
        methods: HashMap<String, Rc<Function>>,
        statics: HashMap<String, Rc<Function>>,
        super_class: Option<Rc<Class>>,
        init: Option<Rc<Function>>,
    ) -> Self {
        return Self {
            name,
            methods,
            statics,
            super_class,
            init,
        };
    }

    pub fn get_method(&self, name: &str) -> Option<Rc<Function>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }

        if let Some(sc) = self.super_class.as_ref() {
            return sc.get_method(name);
        }

        None
    }

    pub fn get_static(&self, name: &str) -> Option<Rc<Function>> {
        self.statics.get(name).map(|f| f.clone())
    }

    pub fn init(&self) -> Option<Rc<Function>> {
        self.init.clone()
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[class {}]", self.name)
    }
}

#[derive(Debug)]
pub struct ClassInstance {
    constructor: Rc<Class>,
    properties: HashMap<String, LoxObject>,
}

impl ClassInstance {
    pub fn new(constructor: Rc<Class>) -> Self {
        return Self {
            constructor,
            properties: HashMap::with_capacity(DEFAULT_PROPERTY_HASH_SIZE),
        };
    }

    pub fn new_lox_object(constructor: Rc<Class>) -> LoxObject {
        LoxObject::ClassInstance(Rc::new(RefCell::new(Self::new(constructor))))
    }

    pub fn get(&self, prop: &str) -> Option<LoxObject> {
        if let Some(obj) = self.properties.get(prop) {
            return Some(obj.clone());
        }

        if let Some(func) = self.constructor.get_method(prop) {
            return Some(func.into());
        }

        None
    }

    pub fn set(&mut self, prop: &str, value: LoxObject) -> Option<LoxObject> {
        self.properties.insert(prop.to_string(), value)
    }

    pub fn init(&self) -> Option<Rc<Function>> {
        self.constructor.init()
    }
}

impl fmt::Display for ClassInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{}}", self.constructor.name)
    }
}
