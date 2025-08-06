use super::class::ClassInstance;
use super::object::LoxObject;
use super::scope::Scope;
use crate::lang::tree::ast::Stmt;
use std::cell::RefCell;
use std::fmt;
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

    pub fn bind(&self, target: LoxObject) -> Self {
        let mut env = Scope::from(self.closure.clone());
        env.declare("this");
        env.define("this", target);
        Self::new(
            Rc::new(RefCell::new(env)),
            self.params.clone(),
            self.body.clone(),
        )
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function(")?;
        let max_len = self.params.len();
        if max_len == 0 {
            return write!(f, ") {{}}");
        }
        write!(f, "{}", self.params()[0])?;
        for param in &self.params()[1..max_len.min(3)] {
            write!(f, ", {}", param)?;
        }
        if max_len > 3 {
            return write!(f, ", ...) {{}}");
        } else {
            return write!(f, ") {{}}");
        }
    }
}
