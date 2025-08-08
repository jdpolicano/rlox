use crate::interpreter::runtime::class::{Class, ClassInstance};
use crate::interpreter::runtime::error::{BinaryError, LoxError, RuntimeError};
use crate::interpreter::runtime::eval::{Eval, EvalResult};
use crate::interpreter::runtime::function::Function;
use crate::interpreter::runtime::native::setup_native;
use crate::interpreter::runtime::object::LoxObject;
use crate::interpreter::runtime::scope::Scope;
use crate::lang::tree::ast::{
    self, BinaryOperator, Callee, Expr, Identifier, Literal, LogicalOperator, Stmt, UnaryPrefix,
};
use crate::lang::visitor::Visitor;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

    fn declare(&mut self, name: &str) -> usize {
        self.current_scope.borrow_mut().declare(name)
    }

    fn define(&mut self, name: &str, value: LoxObject) {
        self.current_scope.borrow_mut().define(name, value);
    }

    fn bind(&mut self, ident: &Identifier, value: LoxObject) {
        // 2. If resolver gave us a (depth,slot), it's a local…
        if let Some(_) = ident.depth_slot() {
            self.declare(ident.name_str());
            self.define(ident.name_str(), value)
        } else {
            // …otherwise it's a global
            self.set_global(ident.name_str(), value);
        }
    }

    fn set_at(&mut self, distance: usize, slot: usize, value: LoxObject) {
        self.current_scope
            .borrow_mut()
            .set_at(distance, slot, value);
    }

    fn get_at(&self, distance: usize, slot: usize) -> LoxObject {
        self.current_scope.borrow().get_at(distance, slot)
    }

    pub fn get_global(&self, name: &str) -> Option<LoxObject> {
        self.globals.get(name).map(|v| v.clone())
    }

    pub fn set_global(&mut self, name: &str, value: LoxObject) {
        self.globals.insert(name.to_string(), value);
    }

    pub fn assign_global(
        &mut self,
        name_ident: &Identifier,
        value: LoxObject,
    ) -> Result<(), RuntimeError> {
        let key = name_ident.name_str();
        if !self.globals.contains_key(key) {
            return Err(reference_error(name_ident));
        }
        self.set_global(key, value);
        Ok(())
    }

    pub fn resolve(&self, name: &Identifier) -> Option<LoxObject> {
        if let Some((depth, slot)) = name.depth_slot() {
            Some(self.get_at(depth, slot))
        } else {
            self.get_global(name.name_str())
        }
    }

    fn create_scope(&mut self) {
        let next = Scope::from(self.current_scope.clone());
        self.current_scope = Rc::new(RefCell::new(next));
    }

    fn shed_scope(&mut self) {
        let parent = self.current_scope.borrow().parent();
        if let Some(p) = parent {
            self.current_scope = p
        }
    }

    fn call_fn(&mut self, func: &Function, args: Vec<LoxObject>) -> EvalResult {
        // copy our current scope.
        let original = self.current_scope.clone();
        // setup the environment for the func's enclosing scope.
        self.current_scope = func.closure();
        // setup a fresh environment for the parameters to be bound to the arguments.
        self.create_scope();
        // setup the stack local arguments.
        self.setup_fn_stack(func, args);
        // call the function
        let eval = func.body().accept(self);
        // peel off the parameter's scope
        self.shed_scope();
        //println!("scope after calling func \n{:#?}", self.current_scope);
        // return to our original state.
        self.current_scope = original;
        eval
    }

    // it is the responsibliity of the caller to have properly set up the state
    // for local variables.
    fn setup_fn_stack(&mut self, func: &Function, args: Vec<LoxObject>) {
        let params = func.params();
        if params.len() == 0 {
            return;
        }
        for param in params {
            self.declare(param);
        }
        let pairs = params.iter().zip(args.into_iter());
        for (name, value) in pairs {
            self.define(name, value);
        }
    }

    fn handle_object_get(&mut self, obj: LoxObject, property: &Identifier) -> EvalResult {
        match obj {
            LoxObject::ClassInstance(ci) => self.handle_class_instance_get(ci, property),
            LoxObject::Class(c) => self.handle_class_get(c, property),
            _ => Err(reference_error(property)),
        }
    }

    fn handle_class_instance_get(
        &mut self,
        ci: Rc<RefCell<ClassInstance>>,
        property: &Identifier,
    ) -> EvalResult {
        if let Some(v) = ci.borrow().get(property.name_str()) {
            match v {
                LoxObject::Function(func) => {
                    let obj = LoxObject::ClassInstance(ci.clone());
                    let bound_func = func.bind(obj);
                    Ok(LoxObject::from(bound_func).into())
                }
                _ => Ok(v.clone().into()),
            }
        } else {
            Err(ref_error_prop_access(property))
        }
    }

    fn handle_class_get(&mut self, class: Rc<Class>, property: &Identifier) -> EvalResult {
        if let Some(v) = class.get_static(property.name_str()) {
            match v {
                LoxObject::Function(func) => Ok(LoxObject::from(func.clone()).into()),
                _ => Ok(v.clone().into()),
            }
        } else {
            Err(ref_error_prop_access(property))
        }
    }
}

impl Visitor<EvalResult, Expr, Stmt> for Lox {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> EvalResult {
        let l = unwrap_to_object(left.accept(self)?).map_err(|e| e.with_place(op.position()))?;
        let r = unwrap_to_object(right.accept(self)?).map_err(|e| e.with_place(op.position()))?;
        match binary_op(&l, &r, op) {
            Ok(v) => Ok(v.into()),
            Err(err_type) => Err(binary_op_error(&l, &r, op, err_type)),
        }
    }

    fn visit_logical(&mut self, left: &Expr, op: LogicalOperator, right: &Expr) -> EvalResult {
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

    fn visit_grouping(&mut self, expr: &Expr) -> EvalResult {
        expr.accept(self)
    }

    fn visit_literal(&mut self, value: &Literal) -> EvalResult {
        Ok(LoxObject::from(value).into())
    }

    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> EvalResult {
        let eval = expr.accept(self)?;
        let value = unwrap_to_object(eval).map_err(|e| e.with_place(prefix.position()))?;
        match unary_op(&value, prefix) {
            Ok(v) => Ok(v.into()),
            Err(_) => Err(unary_prefix_error(&value, prefix)),
        }
    }

    fn visit_variable(&mut self, ident: &Identifier) -> EvalResult {
        let v = if let Some((depth, slot)) = ident.depth_slot() {
            self.get_at(depth, slot).into()
        } else {
            self.get_global(ident.name_str())
                .ok_or_else(|| reference_error(ident))?
        };
        Ok(v.into())
    }

    fn visit_assignment(&mut self, ident: &Identifier, value: &Expr) -> EvalResult {
        let eval = value.accept(self)?;
        let value = unwrap_to_object(eval).map_err(|e| e.with_place(ident.position()))?;
        // println!("ident is {:#?}", ident);
        if let Some((depth, slot)) = ident.depth_slot() {
            self.set_at(depth, slot, value.clone());
            return Ok(value.into());
        } else {
            return self
                .assign_global(ident, value.clone())
                .map(|_| Eval::from(value));
        };
    }

    fn visit_call(&mut self, callee: &Callee, args: &[Expr]) -> EvalResult {
        let eval = callee.expr.accept(self)?;
        let call_obj = unwrap_to_object(eval).map_err(|e| e.with_place(callee.position()))?;
        let mut rt_args = Vec::with_capacity(args.len());
        for arg in args {
            let eval = arg.accept(self)?;
            let obj = unwrap_to_object(eval).map_err(|e| e.with_place(callee.position()))?;
            rt_args.push(obj)
        }
        match call_obj {
            LoxObject::Native(f) => f(self, rt_args).map_err(|e| e.with_place(callee.position())),
            LoxObject::Function(f) => self
                .call_fn(f.as_ref(), rt_args)
                .map(|v| v.unwrap_return())
                .map_err(|e| e.with_place(callee.position())),
            LoxObject::Class(c) => {
                let instance = ClassInstance::new(c);
                if let Some(init) = instance.init() {
                    let obj = LoxObject::from(instance);
                    let _ = self
                        .call_fn(&init.bind(obj.clone()), rt_args)
                        .map_err(|e| e.with_place(callee.position()))?;
                    Ok(obj.into())
                } else {
                    Ok(LoxObject::from(instance).into())
                }
            }
            _ => Err(
                RuntimeError::from(type_error("function", call_obj.type_str()))
                    .with_place(callee.position()),
            ),
        }
    }

    fn visit_function(&mut self, value: &ast::Function) -> EvalResult {
        Ok(LoxObject::from(Function::new(
            self.current_scope.clone(),
            value
                .params()
                .iter()
                .map(|p| p.name_str().to_string())
                .collect(),
            value.body(),
        ))
        .into())
    }
    fn visit_get(&mut self, object: &Expr, property: &Identifier) -> EvalResult {
        let obj = object.accept(self)?;
        match obj {
            Eval::Object(obj) => self.handle_object_get(obj, property),
            _ => Err(type_error("class instance", obj.type_str())),
        }
    }

    fn visit_set(&mut self, object: &Expr, property: &Identifier, value: &Expr) -> EvalResult {
        let obj = object.accept(self)?;
        match obj {
            Eval::Object(LoxObject::ClassInstance(ci)) => {
                let eval = value.accept(self)?;
                let value =
                    unwrap_to_object(eval).map_err(|e| e.with_place(property.position()))?;
                ci.borrow_mut().set(property.name_str(), value);
                Ok(Eval::new_nil())
            }
            _ => Err(type_error("class instance", obj.type_str())),
        }
    }

    fn visit_this(&mut self, ident: &Identifier) -> EvalResult {
        match self.resolve(ident) {
            Some(v) => Ok(Eval::from(v)),
            _ => Err(reference_error(ident)),
        }
    }

    fn visit_break_statement(&mut self) -> EvalResult {
        Ok(Eval::new_break())
    }

    fn visit_continue_statment(&mut self) -> EvalResult {
        Ok(Eval::new_continue().into())
    }

    fn visit_return_statment(&mut self, value: Option<&Expr>) -> EvalResult {
        if let Some(v) = value {
            let eval = v.accept(self)?;
            let obj = unwrap_to_object(eval)?;
            return Ok(Eval::new_return(obj));
        }
        Ok(Eval::new_return(LoxObject::new_nil()))
    }

    fn visit_expression_statement(&mut self, expr: &Expr) -> EvalResult {
        expr.accept(self)
    }

    fn visit_print_statement(&mut self, expr: &Expr) -> EvalResult {
        let v = expr.accept(self)?;
        v.with_object(|obj| println!("{}", obj));
        Ok(v)
    }

    fn visit_var_statement(
        &mut self,
        ident: &Identifier,
        initializer: Option<&Expr>,
    ) -> EvalResult {
        // 1. Evaluate the initializer (or nil)
        let value = if let Some(expr) = initializer {
            unwrap_to_object(expr.accept(self)?)?
        } else {
            LoxObject::new_nil()
        };
        self.bind(ident, value);
        Ok(Eval::new_nil())
    }

    fn visit_block_statement(&mut self, statments: &[Stmt]) -> EvalResult {
        // create a new scope
        self.create_scope();
        let mut ret = Eval::new_nil();
        for stmt in statments {
            let v = stmt.accept(self)?;
            if v.is_control() {
                ret = v;
                break;
            }
        }
        // get rid of the temporary scope we created.
        self.shed_scope();
        Ok(ret)
    }

    fn visit_if_statement(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: Option<&Stmt>,
    ) -> EvalResult {
        if condition.accept(self)?.truthy() {
            if_block.accept(self)
        } else if let Some(else_block) = else_block {
            else_block.accept(self)
        } else {
            Ok(Eval::new_nil())
        }
    }

    fn visit_while_statement(&mut self, condition: &Expr, block: &Stmt) -> EvalResult {
        while condition.accept(self)?.truthy() {
            let v = block.accept(self)?;
            if v.is_break() {
                break;
            }
            if v.is_return() {
                return Ok(v);
            }
        }
        Ok(LoxObject::new_nil().into())
    }

    // todo: should this just be desugared into a var statement?
    // I want to wait to see if this is the exact same logic or not.
    fn visit_class_statement(
        &mut self,
        name: &Identifier,
        methods: &[ast::Function],
    ) -> EvalResult {
        let mut class_methods = HashMap::with_capacity(methods.len());
        let mut static_methods = HashMap::with_capacity(methods.len());
        let mut init = None;
        for method in methods {
            // the parser should have already confirmed that this is safe.
            let name = method.name().unwrap().name_str().to_string();
            let func = Function::new(
                self.current_scope.clone(),
                method.param_strings(),
                method.body(),
            );

            // todo: parser should ensure that there are no "static" init functions.
            if name == "init" {
                init.replace(LoxObject::from(func));
            } else if method.is_static() {
                static_methods.insert(name, LoxObject::from(func));
            } else {
                class_methods.insert(name, LoxObject::from(func));
            }
        }
        let class_name = String::from(name.name_str());
        let class = LoxObject::from(Class::new(class_name, class_methods, static_methods, init));
        self.bind(name, class.clone());
        Ok(Eval::Object(class))
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
        BinaryOperator::Star { .. } => apply_math_op(l, r, |a, b| a * b),
        BinaryOperator::Greater { .. } => apply_comparison(l, r, |a, b| a > b),
        BinaryOperator::GreaterEqual { .. } => apply_comparison(l, r, |a, b| a >= b),
        BinaryOperator::Less { .. } => apply_comparison(l, r, |a, b| a < b),
        BinaryOperator::LessEqual { .. } => apply_comparison(l, r, |a, b| a <= b),
        BinaryOperator::Equal { .. } => Ok(LoxObject::from(l == r)),
        BinaryOperator::NotEqual { .. } => Ok(LoxObject::from(l != r)),
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
) -> RuntimeError {
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

    RuntimeError::from(LoxError::TypeError(msg)).with_place(op.position())
}

fn unary_prefix_error(l: &LoxObject, prefix: UnaryPrefix) -> RuntimeError {
    let msg = format!("invalid type {} for prefix {}", l.type_str(), prefix);
    RuntimeError::from(LoxError::TypeError(msg)).with_place(prefix.position())
}

fn reference_error(ident: &Identifier) -> RuntimeError {
    let msg = format!("undeclared identifier '{}'", ident.name_str());
    RuntimeError::from(LoxError::ReferenceError(msg)).with_place(ident.position())
}

fn ref_error_prop_access(ident: &Identifier) -> RuntimeError {
    let msg = format!("undefined property '{}'", ident.name_str());
    RuntimeError::from(LoxError::ReferenceError(msg)).with_place(ident.position())
}

fn type_error(expected: &str, recieved: &str) -> RuntimeError {
    LoxError::TypeError(format!(
        "expected type '{}' but recieved {}",
        expected, recieved
    ))
    .into()
}

fn unwrap_to_object(eval: Eval) -> Result<LoxObject, RuntimeError> {
    match eval {
        Eval::Object(obj) => Ok(obj),
        _ => Err(type_error("object", eval.type_str())),
    }
}
