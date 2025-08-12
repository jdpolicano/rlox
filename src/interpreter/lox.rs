use crate::interpreter::helpers::{
    binary_op, binary_op_error, ref_error_prop_access, ref_error_prop_not_obj, reference_error,
    type_error, unary_op, unary_prefix_error, unwrap_to_object,
};
use crate::interpreter::runtime::class::{Class, ClassInstance};
use crate::interpreter::runtime::error::RuntimeError;
use crate::interpreter::runtime::eval::{Eval, EvalResult};
use crate::interpreter::runtime::function::Function;
use crate::interpreter::runtime::native::setup_native;
use crate::interpreter::runtime::object::LoxObject;
use crate::interpreter::runtime::scope::Scope;
use crate::lang::tokenizer::span::Span;
use crate::lang::tree::ast::{
    self, BinaryOperator, Callee, Expr, Identifier, Literal, LogicalOperator, UnaryPrefix,
};
use crate::lang::tree::ast::{PropertyName, Stmt};
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

    pub fn declare(&mut self, name: &str) -> usize {
        self.current_scope.borrow_mut().declare(name)
    }

    pub fn define(&mut self, name: &str, value: LoxObject) {
        self.current_scope.borrow_mut().define(name, value);
    }

    pub fn bind(&mut self, ident: &Identifier, value: LoxObject) {
        if let Some(_) = ident.depth_slot() {
            self.declare(ident.name_str());
            self.define(ident.name_str(), value)
        } else {
            self.set_global(ident.name_str(), value);
        }
    }

    pub fn set_at(&mut self, distance: usize, slot: usize, value: LoxObject) {
        self.current_scope
            .borrow_mut()
            .set_at(distance, slot, value);
    }

    pub fn get_at(&self, distance: usize, slot: usize) -> LoxObject {
        self.current_scope.borrow().get_at(distance, slot)
    }

    pub fn get_global(&self, name: &str) -> Option<LoxObject> {
        self.globals.get(name).cloned()
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

    pub fn create_scope(&mut self) {
        self.current_scope = Rc::new(RefCell::new(Scope::from(self.current_scope.clone())));
    }

    pub fn shed_scope(&mut self) {
        let parent = self.current_scope.borrow().parent();
        if let Some(parent) = parent {
            self.current_scope = parent;
        }
    }

    pub fn call_fn(&mut self, func: &Function, args: Vec<LoxObject>) -> EvalResult {
        let original_scope = self.current_scope.clone();
        self.setup_function_environment(func, args)?;
        let eval = func.body().accept(self);
        self.restore_scope(original_scope);
        eval
    }

    pub fn setup_function_environment(
        &mut self,
        func: &Function,
        args: Vec<LoxObject>,
    ) -> Result<(), RuntimeError> {
        self.current_scope = func.closure();
        self.create_scope();
        self.setup_fn_stack(func, args);
        Ok(())
    }

    pub fn restore_scope(&mut self, original_scope: Rc<RefCell<Scope>>) {
        self.shed_scope();
        self.current_scope = original_scope;
    }

    pub fn setup_fn_stack(&mut self, func: &Function, args: Vec<LoxObject>) {
        let params = func.params();
        for param in params {
            self.declare(param);
        }
        for (name, value) in params.iter().zip(args) {
            self.define(name, value);
        }
    }

    pub fn handle_object_get(&mut self, obj: LoxObject, property: &PropertyName) -> EvalResult {
        match obj {
            LoxObject::ClassInstance(ci) => self.handle_class_instance_get(ci, property),
            LoxObject::Class(c) => self.handle_class_get(c, property),
            _ => Err(ref_error_prop_not_obj(property, obj.type_str())),
        }
    }

    pub fn handle_class_instance_get(
        &mut self,
        ci: Rc<RefCell<ClassInstance>>,
        property: &PropertyName,
    ) -> EvalResult {
        if let Some(value) = ci.borrow().get(property.name_str()) {
            Ok(match value {
                LoxObject::Function(func) => {
                    let bound_func = func.bind(LoxObject::ClassInstance(ci.clone()));
                    LoxObject::from(bound_func).into()
                }
                _ => value.clone().into(),
            })
        } else {
            Err(ref_error_prop_access(property))
        }
    }

    pub fn handle_class_get(&mut self, class: Rc<Class>, property: &PropertyName) -> EvalResult {
        if let Some(value) = class.get_static(property.name_str()) {
            Ok(Eval::Object(LoxObject::Function(value)))
        } else {
            Err(ref_error_prop_access(property))
        }
    }

    pub fn resolve_variable(&self, ident: &Identifier) -> EvalResult {
        if let Some((depth, slot)) = ident.depth_slot() {
            Ok(self.get_at(depth, slot).into())
        } else {
            self.get_global(ident.name_str())
                .ok_or_else(|| reference_error(ident))
                .map(|v| v.into())
        }
    }

    pub fn evaluate_arguments(&mut self, args: &[Expr]) -> Result<Vec<LoxObject>, RuntimeError> {
        args.iter()
            .map(|arg| {
                arg.accept(self).and_then(|eval| {
                    unwrap_to_object(eval).map_err(|e| RuntimeError::new(e, arg.span()))
                })
            })
            .collect()
    }

    pub fn execute_call(
        &mut self,
        call_obj: LoxObject,
        rt_args: Vec<LoxObject>,
        span: Span,
    ) -> EvalResult {
        match call_obj {
            LoxObject::Native(f) => f(self, rt_args).map_err(|e| RuntimeError::new(e, span)),
            LoxObject::Function(f) => self
                .call_fn(f.as_ref(), rt_args)
                .map(|v| v.unwrap_return())
                .into(),
            LoxObject::Class(c) => self.instantiate_class(c, rt_args),
            _ => Err(RuntimeError::new(
                type_error("function", call_obj.type_str()),
                span,
            )),
        }
    }

    pub fn instantiate_class(&mut self, class: Rc<Class>, rt_args: Vec<LoxObject>) -> EvalResult {
        let instance = ClassInstance::new(class);
        if let Some(init) = instance.init() {
            let obj = LoxObject::from(instance);
            self.call_fn(&init.bind(obj.clone()), rt_args)?;
            Ok(obj.into())
        } else {
            Ok(LoxObject::from(instance).into())
        }
    }

    pub fn execute_block(&mut self, statements: &[Stmt]) -> EvalResult {
        for stmt in statements {
            let result = stmt.accept(self)?;
            if result.is_control() {
                return Ok(result);
            }
        }
        Ok(Eval::new_nil())
    }

    pub fn collect_class_methods(
        &mut self,
        methods: &[ast::Function],
    ) -> (
        HashMap<String, Rc<Function>>,
        HashMap<String, Rc<Function>>,
        Option<Rc<Function>>,
    ) {
        let mut class_methods = HashMap::new();
        let mut static_methods = HashMap::new();
        let mut init = None;

        for method in methods {
            let name = method.name().unwrap().name_str().to_string();
            let func = Function::new(
                self.current_scope.clone(),
                method.param_strings(),
                method.body(),
            );

            if name == "init" {
                init = Some(Rc::new(func));
            } else if method.is_static() {
                static_methods.insert(name, Rc::new(func));
            } else {
                class_methods.insert(name, Rc::new(func));
            }
        }

        (class_methods, static_methods, init)
    }

    pub fn get_super_class(
        &mut self,
        variable: Option<&Expr>,
    ) -> Result<Option<Rc<Class>>, RuntimeError> {
        if let Some(expr) = variable {
            match expr.accept(self)? {
                Eval::Object(LoxObject::Class(c)) => {
                    return Ok(Some(c));
                }
                other => {
                    return Err(RuntimeError::new(
                        type_error("class declaration", other.type_str()),
                        expr.span(),
                    ));
                }
            }
        }
        Ok(None)
    }
}

impl Visitor<EvalResult, Expr, Stmt> for Lox {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> EvalResult {
        let l = left.accept(self)?;
        let r = right.accept(self)?;
        let lobj = unwrap_to_object(l).map_err(|e| RuntimeError::new(e, left.span()))?;
        let robj = unwrap_to_object(r).map_err(|e| RuntimeError::new(e, right.span()))?;
        binary_op(&lobj, &robj, op).map_or_else(
            |err_type| Err(binary_op_error(&lobj, &robj, op, err_type)),
            |v| Ok(v.into()),
        )
    }

    fn visit_logical(&mut self, left: &Expr, op: LogicalOperator, right: &Expr) -> EvalResult {
        let lhs = left.accept(self)?;
        match op {
            LogicalOperator::And(_) if !lhs.truthy() => Ok(lhs),
            LogicalOperator::Or(_) if lhs.truthy() => Ok(lhs),
            _ => right.accept(self),
        }
    }

    fn visit_grouping(&mut self, expr: &Expr) -> EvalResult {
        expr.accept(self)
    }

    fn visit_literal(&mut self, value: &Literal) -> EvalResult {
        Ok(LoxObject::from(value).into())
    }

    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> EvalResult {
        let eval = expr.accept(self)?;
        let value = unwrap_to_object(eval).map_err(|e| RuntimeError::new(e, expr.span()))?;
        unary_op(&value, prefix).map_or_else(
            |_| Err(unary_prefix_error(&value, prefix)),
            |v| Ok(v.into()),
        )
    }

    fn visit_variable(&mut self, ident: &Identifier) -> EvalResult {
        self.resolve_variable(ident)
    }

    fn visit_assignment(&mut self, ident: &Identifier, value: &Expr) -> EvalResult {
        let eval = value.accept(self)?;
        let value = unwrap_to_object(eval).map_err(|e| RuntimeError::new(e, value.span()))?;
        if let Some((depth, slot)) = ident.depth_slot() {
            self.set_at(depth, slot, value.clone());
            Ok(value.into())
        } else {
            self.assign_global(ident, value.clone())
                .map(|_| Eval::from(value))
        }
    }

    fn visit_call(&mut self, callee: &Callee, args: &[Expr]) -> EvalResult {
        let eval = callee.expr.accept(self)?;
        let call_obj = unwrap_to_object(eval).map_err(|e| RuntimeError::new(e, callee.span()))?;
        let rt_args = self.evaluate_arguments(args)?;
        self.execute_call(call_obj, rt_args, callee.span())
    }

    fn visit_function(&mut self, value: &ast::Function) -> EvalResult {
        Ok(LoxObject::from(Function::new(
            self.current_scope.clone(),
            value.param_strings(),
            value.body(),
        ))
        .into())
    }

    fn visit_get(&mut self, object: &Expr, property: &PropertyName) -> EvalResult {
        let obj = object.accept(self)?;
        match obj {
            Eval::Object(obj) => self.handle_object_get(obj, property),
            _ => Err(RuntimeError::new(
                type_error("class instance", obj.type_str()),
                object.span(),
            )),
        }
    }

    fn visit_set(&mut self, object: &Expr, property: &PropertyName, value: &Expr) -> EvalResult {
        let obj = object.accept(self)?;
        match obj {
            Eval::Object(LoxObject::ClassInstance(ci)) => {
                let eval = value.accept(self)?;
                let value =
                    unwrap_to_object(eval).map_err(|e| RuntimeError::new(e, value.span()))?;
                ci.borrow_mut().set(property.name_str(), value);
                Ok(Eval::new_nil())
            }
            _ => Err(RuntimeError::new(
                type_error("class instance", obj.type_str()),
                object.span(),
            )),
        }
    }

    fn visit_this(&mut self, ident: &Identifier) -> EvalResult {
        self.resolve(ident)
            .map(Eval::from)
            .ok_or_else(|| reference_error(ident))
    }

    fn visit_break_statement(&mut self) -> EvalResult {
        Ok(Eval::new_break())
    }

    fn visit_continue_statment(&mut self) -> EvalResult {
        Ok(Eval::new_continue().into())
    }

    fn visit_return_statment(&mut self, value: Option<&Expr>) -> EvalResult {
        if let Some(expr) = value {
            let v = expr.accept(self)?;
            let obj = unwrap_to_object(v).map_err(|e| RuntimeError::new(e, expr.span()))?;
            Ok(Eval::new_return(obj))
        } else {
            Ok(Eval::new_return(LoxObject::new_nil()))
        }
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
        let value = if let Some(init) = initializer {
            let v = init.accept(self)?;
            unwrap_to_object(v).map_err(|e| RuntimeError::new(e, init.span()))?
        } else {
            LoxObject::new_nil()
        };
        self.bind(ident, value);
        Ok(Eval::new_nil())
    }

    fn visit_block_statement(&mut self, statements: &[Stmt]) -> EvalResult {
        self.create_scope();
        let result = self.execute_block(statements);
        self.shed_scope();
        result
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
            let result = block.accept(self)?;
            if result.is_break() {
                break;
            }
            if result.is_return() {
                return Ok(result);
            }
        }
        Ok(LoxObject::new_nil().into())
    }

    fn visit_class_statement(
        &mut self,
        name: &Identifier,
        super_class: Option<&Expr>,
        methods: &[ast::Function],
    ) -> EvalResult {
        let super_class = self.get_super_class(super_class)?;
        let (class_methods, static_methods, init) = self.collect_class_methods(methods);
        let class_name = name.name_str().to_string();
        let class = Class::new(class_name, class_methods, static_methods, super_class, init);
        let obj = LoxObject::from(class);
        self.bind(name, obj.clone());
        Ok(Eval::Object(obj))
    }
}
