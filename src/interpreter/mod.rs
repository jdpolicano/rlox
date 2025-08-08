use crate::interpreter::runtime::error::RuntimeError;
use crate::interpreter::runtime::native::setup_native;
use crate::interpreter::runtime::object::LoxObject;
use crate::interpreter::runtime::scope::Scope;
use crate::lang::tree::ast::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(self) mod lox;
pub(self) mod runtime;

pub struct Lox {
    globals: HashMap<String, LoxObject>,
    current_scope: Rc<RefCell<Scope>>,
}

impl Lox {
    pub fn new() -> Self {
        let mut me = Self {
            globals: HashMap::new(),
            current_scope: Rc::new(RefCell::new(Scope::default())),
        };
        setup_native(&mut me);
        me
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in statements {
            let _ = stmt.accept(self)?;
        }
        Ok(())
    }
}
