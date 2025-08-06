use super::class::{Class, ClassInstance};
use super::function::Function;
use super::native::NativeFn;
use super::primitive::Primitive;
use crate::lang::tree::ast;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LoxObject {
    Primitive(Primitive),
    Class(Rc<Class>),
    ClassInstance(Rc<RefCell<ClassInstance>>),
    Function(Rc<Function>),
    Native(NativeFn),
}

impl From<ast::Literal> for LoxObject {
    fn from(value: ast::Literal) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<&ast::Literal> for LoxObject {
    fn from(value: &ast::Literal) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<f64> for LoxObject {
    fn from(value: f64) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<bool> for LoxObject {
    fn from(value: bool) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<&str> for LoxObject {
    fn from(value: &str) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<(&str, &str)> for LoxObject {
    fn from(value: (&str, &str)) -> Self {
        let mut container = String::with_capacity(value.0.len() + value.1.len());
        container.push_str(value.0);
        container.push_str(value.1);
        Self::Primitive(container.into())
    }
}

impl From<Function> for LoxObject {
    fn from(value: Function) -> Self {
        Self::Function(Rc::new(value))
    }
}

impl From<Class> for LoxObject {
    fn from(value: Class) -> Self {
        LoxObject::Class(Rc::new(value))
    }
}

impl From<Rc<Class>> for LoxObject {
    fn from(value: Rc<Class>) -> Self {
        LoxObject::Class(value)
    }
}

impl From<ClassInstance> for LoxObject {
    fn from(value: ClassInstance) -> Self {
        LoxObject::ClassInstance(Rc::new(RefCell::new(value)))
    }
}

impl fmt::Display for LoxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxObject::Primitive(prim) => write!(f, "{}", prim),
            LoxObject::Function(func) => write!(f, "{}", func),
            LoxObject::Native(_) => write!(f, "[native]()"),
            LoxObject::Class(c) => write!(f, "{}", c),
            LoxObject::ClassInstance(i) => write!(f, "{}", i.borrow()),
        }
    }
}

impl PartialEq for LoxObject {
    fn eq(&self, other: &LoxObject) -> bool {
        match (self, other) {
            (LoxObject::Primitive(a), LoxObject::Primitive(b)) => a.eq(b),
            (LoxObject::Function(f1), LoxObject::Function(f2)) => Rc::ptr_eq(f1, f2),
            (LoxObject::Class(c1), LoxObject::Class(c2)) => Rc::ptr_eq(c1, c2),
            (LoxObject::ClassInstance(c1), LoxObject::ClassInstance(c2)) => Rc::ptr_eq(c1, c2),
            // function pointers are not guarranteed to have a consistent memory address
            // see: https://doc.rust-lang.org/nightly/core/ptr/fn.fn_addr_eq.html
            //
            // However, I think that because of the way we have implemented native functions as a
            // function pointer that is created - and bound - only once on runtime startup,
            // we are always copying that address by value if we assign some expression to it.
            (LoxObject::Native(f1), LoxObject::Native(f2)) => std::ptr::fn_addr_eq(*f1, *f2),
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl LoxObject {
    pub fn new_nil() -> Self {
        Self::Primitive(Primitive::Nil)
    }

    pub fn is_number(&self) -> bool {
        match self {
            LoxObject::Primitive(Primitive::Number(_)) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            LoxObject::Primitive(Primitive::String(_)) => true,
            _ => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            LoxObject::Primitive(Primitive::Boolean(_)) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LoxObject::Primitive(Primitive::Nil) => true,
            _ => false,
        }
    }

    pub fn is_function(&self) -> bool {
        match self {
            LoxObject::Function { .. } => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        if let LoxObject::Primitive(Primitive::Number(n)) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let LoxObject::Primitive(Primitive::String(s)) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let LoxObject::Primitive(Primitive::Boolean(b)) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn as_nil(&self) -> Option<()> {
        if let LoxObject::Primitive(Primitive::Nil) = self {
            Some(())
        } else {
            None
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            LoxObject::Primitive(prim) => prim.truthy(),
            _ => false,
        }
    }

    pub fn type_str(&self) -> &str {
        match self {
            LoxObject::Primitive(p) => p.type_str(),
            LoxObject::Function(_) => "function",
            LoxObject::Native(_) => "native function",
            LoxObject::Class(_) => "class",
            LoxObject::ClassInstance(_) => "class instance",
        }
    }
}
