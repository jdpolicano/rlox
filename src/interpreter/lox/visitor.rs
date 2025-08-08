use crate::interpreter::Lox;
use crate::interpreter::lox::helpers::{
    binary_op, binary_op_error, reference_error, type_error, unary_op, unary_prefix_error,
    unwrap_to_object,
};
use crate::interpreter::runtime::class::Class;
use crate::interpreter::runtime::eval::{Eval, EvalResult};
use crate::interpreter::runtime::function::Function;
use crate::interpreter::runtime::object::LoxObject;
use crate::lang::tree::ast::{
    self, BinaryOperator, Callee, Expr, Identifier, Literal, LogicalOperator, Stmt, UnaryPrefix,
};
use crate::lang::visitor::Visitor;

impl Visitor<EvalResult, Expr, Stmt> for Lox {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> EvalResult {
        let l = unwrap_to_object(left.accept(self)?).map_err(|e| e.with_place(op.position()))?;
        let r = unwrap_to_object(right.accept(self)?).map_err(|e| e.with_place(op.position()))?;
        binary_op(&l, &r, op).map_or_else(
            |err_type| Err(binary_op_error(&l, &r, op, err_type)),
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
        let value = unwrap_to_object(eval).map_err(|e| e.with_place(prefix.position()))?;
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
        let value = unwrap_to_object(eval).map_err(|e| e.with_place(ident.position()))?;
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
        let call_obj = unwrap_to_object(eval).map_err(|e| e.with_place(callee.position()))?;
        let rt_args = self.evaluate_arguments(args, callee.position())?;
        self.execute_call(call_obj, rt_args, callee.position())
    }

    fn visit_function(&mut self, value: &ast::Function) -> EvalResult {
        Ok(LoxObject::from(Function::new(
            self.current_scope.clone(),
            value.param_strings(),
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
        let return_value = value
            .map(|v| v.accept(self).and_then(unwrap_to_object))
            .transpose()?
            .unwrap_or_else(LoxObject::new_nil);
        Ok(Eval::new_return(return_value))
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
        let value = initializer
            .map(|expr| expr.accept(self).and_then(unwrap_to_object))
            .transpose()?
            .unwrap_or_else(LoxObject::new_nil);
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
        methods: &[ast::Function],
    ) -> EvalResult {
        let (class_methods, static_methods, init) = self.collect_class_methods(methods);
        let class_name = name.name_str().to_string();
        let class = LoxObject::from(Class::new(class_name, class_methods, static_methods, init));
        self.bind(name, class.clone());
        Ok(Eval::Object(class))
    }
}
