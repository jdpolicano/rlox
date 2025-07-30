use super::runtime::error::{BinaryError, LoxError};
use super::runtime::scope::Scope;
use crate::interpreter::runtime::value::LoxObject;
use crate::lang::tree::ast::{
    BinaryOperator, Expr, Identifier, Literal, LogicalOperator, Stmt, UnaryPrefix,
};
use crate::lang::visitor::Visitor;
use std::cell::RefCell;
use std::rc::Rc;

// todo: implement lox errors. Should they just be a type of runtime value or should we simply use a result?
type LoxResult = Result<LoxObject, LoxError>;

pub struct Lox {
    current_scope: Rc<RefCell<Scope>>,
    inside_loop: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            current_scope: Rc::new(RefCell::new(Scope::default())),
            inside_loop: false,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), LoxError> {
        for stmt in statements {
            let _ = stmt.accept(self)?;
        }
        Ok(())
    }

    fn create_scope(&mut self) {
        let next = Scope::default().with_parent(self.current_scope.clone());
        self.current_scope = Rc::new(RefCell::new(next));
    }

    fn shed_scope(&mut self) {
        let parent = self.current_scope.borrow().parent();
        if let Some(p) = parent {
            self.current_scope = p
        }
    }

    fn resolve(&self, key: &str) -> Option<LoxObject> {
        self.current_scope.borrow().get(key)
    }

    fn bind_local(&self, key: &str, value: LoxObject) -> Option<LoxObject> {
        self.current_scope.borrow_mut().set_local(key, value)
    }

    fn bind(&self, key: &str, value: LoxObject) -> Option<LoxObject> {
        self.current_scope.borrow_mut().set(key, value)
    }

    fn toggle_loop(&mut self) {
        self.inside_loop = !self.inside_loop;
    }
}

impl Visitor<LoxResult> for Lox {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> LoxResult {
        let l = left.accept(self)?;
        let r = right.accept(self)?;
        match binary_op(&l, &r, op) {
            Ok(v) => Ok(v),
            Err(err_type) => Err(binary_op_error(&l, &r, op, err_type)),
        }
    }

    fn visit_logical(&mut self, left: &Expr, op: LogicalOperator, right: &Expr) -> LoxResult {
        let lhs = left.accept(self)?;
        match op {
            LogicalOperator::And { .. } => {
                if !lhs.truthy() {
                    return Ok(lhs);
                }
            }
            LogicalOperator::Or { .. } => {
                if lhs.truthy() {
                    return Ok(lhs);
                }
            }
        };
        right.accept(self)
    }

    fn visit_grouping(&mut self, expr: &Expr) -> LoxResult {
        expr.accept(self)
    }

    fn visit_literal(&mut self, value: &Literal) -> LoxResult {
        Ok(value.into())
    }

    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> LoxResult {
        let value = expr.accept(self)?;
        match unary_op(&value, prefix) {
            Ok(v) => Ok(v),
            Err(_) => Err(unary_prefix_error(&value, prefix)),
        }
    }

    fn visit_variable(&mut self, ident: &Identifier) -> LoxResult {
        if let Some(v) = self.resolve(ident.name_str()) {
            Ok(v)
        } else {
            Err(reference_error(ident))
        }
    }

    fn visit_assignment(&mut self, ident: &Identifier, value: &Expr) -> LoxResult {
        let value = value.accept(self)?;
        if let Some(_) = self.bind(ident.name_str(), value.clone()) {
            return Ok(value);
        };
        Err(reference_error(ident))
    }

    fn visit_expression_statement(&mut self, expr: &Expr) -> LoxResult {
        expr.accept(self)
    }

    fn visit_print_statement(&mut self, expr: &Expr) -> LoxResult {
        let v = expr.accept(self)?;
        println!("{}", v);
        Ok(v)
    }

    fn visit_var_statement(&mut self, ident: &Identifier, expr: Option<&Expr>) -> LoxResult {
        let value = expr
            .map(|e| e.accept(self))
            .unwrap_or(Ok(LoxObject::new_nil()))?;
        self.bind_local(ident.name_str(), value);
        Ok(LoxObject::new_nil())
    }

    fn visit_block_statement(&mut self, statments: &[Stmt]) -> LoxResult {
        // create a new scope
        self.create_scope();
        let mut ret = LoxObject::new_nil();
        for stmt in statments {
            let v = stmt.accept(self)?;
            if v.is_control() {
                ret = v;
                break;
            }
        }
        self.shed_scope();
        Ok(ret)
    }

    fn visit_if_statement(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: Option<&Stmt>,
    ) -> LoxResult {
        if condition.accept(self)?.truthy() {
            if_block.accept(self)
        } else if let Some(else_block) = else_block {
            else_block.accept(self)
        } else {
            Ok(LoxObject::new_nil())
        }
    }

    fn visit_while_statement(&mut self, condition: &Expr, block: &Stmt) -> LoxResult {
        while condition.accept(self)?.truthy() {
            let v = block.accept(self)?;
            if v.is_break() {
                break;
            }
        }
        Ok(LoxObject::new_nil())
    }

    fn visit_break(&mut self) -> LoxResult {
        Ok(LoxObject::new_break())
    }

    fn visit_continue(&mut self) -> LoxResult {
        Ok(LoxObject::new_continue())
    }
}

fn unary_op(value: &LoxObject, op: UnaryPrefix) -> Result<LoxObject, BinaryError> {
    match op {
        UnaryPrefix::Bang { .. } => Ok(value.truthy().into()),
        UnaryPrefix::Minus { .. } => apply_math_op(value, &(-1.0).into(), |a, b| a * b),
    }
}

fn binary_op(l: &LoxObject, r: &LoxObject, op: BinaryOperator) -> Result<LoxObject, BinaryError> {
    match op {
        // addition is a special case where we need to handle string concatenation.
        BinaryOperator::Plus { .. } => {
            if l.is_number() && r.is_number() {
                apply_math_op(l, r, |a, b| a + b)
            } else {
                concat_strings(l, r)
            }
        }
        BinaryOperator::Minus { .. } => apply_math_op(l, r, |a, b| a - b),
        BinaryOperator::Slash { .. } => apply_math_op(l, r, |a, b| a / b),
        BinaryOperator::Greater { .. } => apply_comparison(l, r, |a, b| a > b),
        BinaryOperator::GreaterEqual { .. } => apply_comparison(l, r, |a, b| a >= b),
        BinaryOperator::Less { .. } => apply_comparison(l, r, |a, b| a < b),
        BinaryOperator::LessEqual { .. } => apply_comparison(l, r, |a, b| a <= b),
        BinaryOperator::Equal { .. } => Ok(LoxObject::from(l == r)),
        BinaryOperator::NotEqual { .. } => Ok(LoxObject::from(l != r)),
        _ => Err(BinaryError::InvalidOperator),
    }
}

fn concat_strings(l: &LoxObject, r: &LoxObject) -> Result<LoxObject, BinaryError> {
    let l_as_str = l.as_string();
    let r_as_str = r.as_string();
    match (l_as_str, r_as_str) {
        (Some(a), Some(b)) => Ok(LoxObject::from((a.as_str(), b.as_str()))),
        // it really doesn't matter what side was a string
        // So just let the user know the right side was different than the left side.
        _ => Err(BinaryError::InvalidTypes),
    }
}

fn apply_math_op<F>(l: &LoxObject, r: &LoxObject, f: F) -> Result<LoxObject, BinaryError>
where
    F: FnOnce(f64, f64) -> f64,
{
    let l_as_num = l.as_number();
    let r_as_num = r.as_number();
    match (l_as_num, r_as_num) {
        (Some(a), Some(b)) => Ok(LoxObject::from(f(a, b))),
        _ => {
            if !l_as_num.is_some() {
                Err(BinaryError::LeftSide)
            } else {
                Err(BinaryError::RightSide)
            }
        }
    }
}

fn apply_comparison<F>(l: &LoxObject, r: &LoxObject, f: F) -> Result<LoxObject, BinaryError>
where
    F: FnOnce(f64, f64) -> bool,
{
    let l_as_num = l.as_number();
    let r_as_num = r.as_number();
    match (l_as_num, r_as_num) {
        (Some(a), Some(b)) => Ok(LoxObject::from(f(a, b))),
        _ => {
            if !l_as_num.is_some() {
                Err(BinaryError::LeftSide)
            } else {
                Err(BinaryError::RightSide)
            }
        }
    }
}

fn binary_op_error(
    l: &LoxObject,
    r: &LoxObject,
    op: BinaryOperator,
    err_type: BinaryError,
) -> LoxError {
    let msg = match err_type {
        BinaryError::LeftSide => format!(
            "lefthand side incorrect type '{}' for op {}",
            l.type_str(),
            op
        ),
        BinaryError::RightSide => format!(
            "righthand side incorrect type '{}' for op {}",
            r.type_str(),
            op
        ),
        BinaryError::InvalidOperator => format!("invalid binary operator {}", op),
        _ => format!("cannot add '{}' + {}'", l.type_str(), r.type_str()),
    };

    LoxError::TypeError {
        msg,
        view: op.view(),
    }
}

fn unary_prefix_error(l: &LoxObject, prefix: UnaryPrefix) -> LoxError {
    let msg = format!("invalid type {} for prefix {}", l.type_str(), prefix);
    LoxError::TypeError {
        msg,
        view: prefix.view(),
    }
}

fn reference_error(ident: &Identifier) -> LoxError {
    LoxError::ReferenceError {
        name: ident.name_str().to_string(),
        view: ident.view(),
    }
}
