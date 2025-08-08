use super::function::Function;
use super::object::LoxObject;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

const DEFAULT_PROPERTY_HASH_SIZE: usize = 16;

#[derive(Debug)]
pub struct Class {
    name: String,
    methods: HashMap<String, LoxObject>,
    statics: HashMap<String, LoxObject>,
    init: Option<LoxObject>,
}

impl Class {
    pub fn new(
        name: String,
        methods: HashMap<String, LoxObject>,
        statics: HashMap<String, LoxObject>,
        init: Option<LoxObject>,
    ) -> Self {
        return Self {
            name,
            methods,
            statics,
            init,
        };
    }

    pub fn get_method(&self, name: &str) -> Option<&LoxObject> {
        self.methods.get(name)
    }

    pub fn get_static(&self, name: &str) -> Option<&LoxObject> {
        self.statics.get(name)
    }

    pub fn init(&self) -> Option<Rc<Function>> {
        if let Some(LoxObject::Function(ref init)) = self.init {
            return Some(init.clone());
        }
        None
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

    pub fn get(&self, prop: &str) -> Option<&LoxObject> {
        self.properties
            .get(prop)
            .or(self.constructor.get_method(prop))
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
