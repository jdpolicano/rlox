use crate::interpreter::Lox;
use crate::interpreter::lox::helpers::{
    ref_error_prop_access, reference_error, type_error, unwrap_to_object,
};
use crate::interpreter::runtime::class::{Class, ClassInstance};
use crate::interpreter::runtime::error::RuntimeError;
use crate::interpreter::runtime::eval::{Eval, EvalResult};
use crate::interpreter::runtime::function::Function;
use crate::interpreter::runtime::object::LoxObject;
use crate::interpreter::runtime::scope::Scope;
use crate::lang::tree::ast::{self, Expr, Identifier, Stmt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

impl Lox {
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

    pub fn handle_object_get(&mut self, obj: LoxObject, property: &Identifier) -> EvalResult {
        match obj {
            LoxObject::ClassInstance(ci) => self.handle_class_instance_get(ci, property),
            LoxObject::Class(c) => self.handle_class_get(c, property),
            _ => Err(reference_error(property)),
        }
    }

    pub fn handle_class_instance_get(
        &mut self,
        ci: Rc<RefCell<ClassInstance>>,
        property: &Identifier,
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

    pub fn handle_class_get(&mut self, class: Rc<Class>, property: &Identifier) -> EvalResult {
        if let Some(value) = class.get_static(property.name_str()) {
            Ok(match value {
                LoxObject::Function(func) => LoxObject::from(func.clone()).into(),
                _ => value.clone().into(),
            })
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

    pub fn evaluate_arguments(
        &mut self,
        args: &[Expr],
        position: usize,
    ) -> Result<Vec<LoxObject>, RuntimeError> {
        args.iter()
            .map(|arg| {
                arg.accept(self)
                    .and_then(|eval| unwrap_to_object(eval).map_err(|e| e.with_place(position)))
            })
            .collect()
    }

    pub fn execute_call(
        &mut self,
        call_obj: LoxObject,
        rt_args: Vec<LoxObject>,
        position: usize,
    ) -> EvalResult {
        match call_obj {
            LoxObject::Native(f) => f(self, rt_args).map_err(|e| e.with_place(position)),
            LoxObject::Function(f) => self
                .call_fn(f.as_ref(), rt_args)
                .map(|v| v.unwrap_return())
                .map_err(|e| e.with_place(position)),
            LoxObject::Class(c) => self.instantiate_class(c, rt_args, position),
            _ => Err(type_error("function", call_obj.type_str()).with_place(position)),
        }
    }

    pub fn instantiate_class(
        &mut self,
        class: Rc<Class>,
        rt_args: Vec<LoxObject>,
        position: usize,
    ) -> EvalResult {
        let instance = ClassInstance::new(class);
        if let Some(init) = instance.init() {
            let obj = LoxObject::from(instance);
            self.call_fn(&init.bind(obj.clone()), rt_args)
                .map_err(|e| e.with_place(position))?;
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
        HashMap<String, LoxObject>,
        HashMap<String, LoxObject>,
        Option<LoxObject>,
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
                init = Some(LoxObject::from(func));
            } else if method.is_static() {
                static_methods.insert(name, LoxObject::from(func));
            } else {
                class_methods.insert(name, LoxObject::from(func));
            }
        }

        (class_methods, static_methods, init)
    }
}
