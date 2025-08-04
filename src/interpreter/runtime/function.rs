use super::scope::Scope;
use crate::lang::tree::ast::Stmt;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Function {
    closure: Rc<RefCell<Scope>>,
    params: Vec<String>,
    body: Rc<Stmt>,
}

impl Function {
    pub fn new(closure: Rc<RefCell<Scope>>, params: Vec<String>, body: Rc<Stmt>) -> Self {
        Self {
            closure,
            params,
            body,
        }
    }

    pub fn body(&self) -> &Stmt {
        self.body.as_ref()
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn params(&self) -> &[String] {
        &self.params[..]
    }

    pub fn closure(&self) -> Rc<RefCell<Scope>> {
        self.closure.clone()
    }
}
