use crate::lang::tree::ast::*;
use crate::lang::visitor::Visitor;
use std::collections::HashMap;

enum FuncType {
    Method,
    Function,
}

/// A Resolver walks your AST **before** runtime and:
/// 1. Assigns each variable use a (depth, slot) pair.
/// 2. Detects reads in their own initializer.
/// 3. Errors on duplicate declarations in the same scope.
#[derive(Debug)]
pub struct Resolver {
    /// Stack of scopes. Each scope maps:
    ///   variable name → (slot index in this frame, is_defined?)
    scopes: Vec<HashMap<String, (usize, bool)>>,
}

impl Resolver {
    /// Create a brand new resolver (no scopes yet).
    pub fn new() -> Self {
        Resolver { scopes: Vec::new() }
    }

    /// Begin a new lexical scope.
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// End the innermost lexical scope.
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    /// Declare a variable in the current scope.
    /// Returns Err if that name is already declared here.
    fn declare(&mut self, name: &Identifier) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.name_str()) {
                // Duplicate var in the same block is an error.
                return Err(format!(
                    "Resolver error: {} already declared in this scope",
                    name
                ));
            }
            // Assign the next available slot (0-based).
            let slot = scope.len();
            // Initially marked "not yet defined" so we catch self-initialization.
            scope.insert(name.to_string(), (slot, false));
        }
        Ok(())
    }

    /// Mark a declared variable as fully initialized.
    fn define(&mut self, name: &Identifier) {
        let depth = self.scopes.len();
        if let Some(scope) = self.scopes.last_mut() {
            if let Some((slot, is_defined)) = scope.get_mut(name.name_str()) {
                name.swap_depth(depth);
                name.swap_slot(*slot);
                *is_defined = true;
            }
        }
    }

    fn put_str(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            let slot = scope.len();
            scope.insert(name.to_string(), (slot, true));
        }
    }

    /// Look up a name through the scope stack.
    /// Returns `Some((depth, (slot, is_defined)))` or `None` if not found.
    fn resolve_local(&self, name: &str) -> Option<(usize, (usize, bool))> {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(&slot_info) = scope.get(name) {
                return Some((depth, slot_info));
            }
        }
        None
    }

    fn resolve_function(&mut self, _: FuncType, value: &Function) -> Result<(), String> {
        // now we begin a scope for local vars.
        self.begin_scope();
        for param in value.params() {
            self.declare(param)?;
            self.define(param);
        }
        value.body().accept(self)?;
        self.end_scope();
        Ok(())
    }
}

impl Visitor<Result<(), String>, Expr, Stmt> for Resolver {
    fn visit_var_statement(
        &mut self,
        ident: &Identifier,
        init: Option<&Expr>,
    ) -> Result<(), String> {
        // 1. Declare (adds slot=false). Errors on duplicate.
        self.declare(ident)?;
        // if there is nothing to initalize with, define the var and move on.
        let expr = match init {
            Some(e) => e,
            _ => {
                self.define(ident);
                return Ok(());
            }
        };
        // else we need to handle some edge cases with functions.
        match expr {
            // named functions can refer to themselves recursively. so we need to define it before
            // we evaluate its body.
            Expr::Function { value } if !value.is_anonymous() => {
                self.define(ident);
                expr.accept(self)?;
                return Ok(());
            }
            // everything else cannot so only define it AFTER we have visited the intializer;
            _ => {
                expr.accept(self)?;
                self.define(ident);
                return Ok(());
            }
        }
    }

    fn visit_variable(&mut self, name: &Identifier) -> Result<(), String> {
        // Attempt to resolve a use of `name`.
        if let Some((depth, (slot, is_defined))) = self.resolve_local(name.name_str()) {
            // If it’s in our current scope (depth==0) but not yet defined, that’s an error.
            if depth == 0 && !is_defined {
                return Err(format!(
                    "Resolver error: cannot read '{}' in its own initializer {}",
                    name.name_str(),
                    name.position()
                ));
            }
            // Store the resolved metadata back into the AST node.
            name.swap_depth(depth);
            name.swap_slot(slot);
        }
        // Otherwise it's a global—interpreter will handle or error later.
        Ok(())
    }

    fn visit_function(&mut self, value: &Function) -> Result<(), String> {
        self.resolve_function(FuncType::Function, value)
    }

    fn visit_assignment(&mut self, name: &Identifier, value: &Expr) -> Result<(), String> {
        // Resolve the value first.
        value.accept(self)?;
        // now figure out if the target is a local or global var
        if let Some((depth, (slot, _))) = self.resolve_local(name.name_str()) {
            // Store the resolved metadata back into the AST node if it was a local var.
            name.swap_depth(depth);
            name.swap_slot(slot);
        }
        Ok(())
    }

    fn visit_print_statement(&mut self, expr: &Expr) -> Result<(), String> {
        expr.accept(self)
    }

    fn visit_expression_statement(&mut self, expr: &Expr) -> Result<(), String> {
        expr.accept(self)
    }

    fn visit_block_statement(&mut self, statements: &[Stmt]) -> Result<(), String> {
        // Every `{` starts a new inner scope.
        self.begin_scope();
        for stmt in statements {
            stmt.accept(self)?;
        }
        self.end_scope();
        Ok(())
    }

    fn visit_if_statement(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<(), String> {
        condition.accept(self)?;
        then_branch.accept(self)?;
        if let Some(else_stmt) = else_branch {
            else_stmt.accept(self)?;
        }
        Ok(())
    }

    fn visit_while_statement(&mut self, condition: &Expr, body: &Stmt) -> Result<(), String> {
        condition.accept(self)?;
        body.accept(self)
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        _operator: BinaryOperator,
        right: &Expr,
    ) -> Result<(), String> {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_logical(
        &mut self,
        left: &Expr,
        _operator: LogicalOperator,
        right: &Expr,
    ) -> Result<(), String> {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Expr) -> Result<(), String> {
        expr.accept(self)
    }

    fn visit_literal(&mut self, _literal: &Literal) -> Result<(), String> {
        Ok(())
    }

    fn visit_unary(&mut self, _operator: UnaryPrefix, expr: &Expr) -> Result<(), String> {
        expr.accept(self)
    }

    fn visit_call(&mut self, callee: &Callee, arguments: &[Expr]) -> Result<(), String> {
        callee.expr.accept(self)?;
        for arg in arguments {
            arg.accept(self)?;
        }
        Ok(())
    }

    fn visit_break_statement(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn visit_continue_statment(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn visit_return_statment(&mut self, value: Option<&Expr>) -> Result<(), String> {
        if let Some(expr) = value {
            expr.accept(self)?;
        }
        Ok(())
    }

    fn visit_class_statement(
        &mut self,
        name: &Identifier,
        super_class: Option<&Expr>,
        methods: &[Function],
    ) -> Result<(), String> {
        self.declare(name)?;
        self.define(name);

        if let Some(sup) = super_class {
            match sup {
                Expr::Variable { value } => {
                    if value.name_str() == name.name_str() {
                        return Err(format!(
                            "super class cannot self reference subclass sub: {} super: {}",
                            name.name_str(),
                            value.name_str()
                        ));
                    }
                }
                _ => {}
            }
            sup.accept(self)?;
        }

        self.begin_scope();
        self.put_str("this");
        for method in methods {
            self.resolve_function(FuncType::Method, method)?;
        }
        self.end_scope();
        Ok(())
    }

    fn visit_get(&mut self, object: &Expr, _property: &Identifier) -> Result<(), String> {
        object.accept(self)
    }

    fn visit_set(
        &mut self,
        object: &Expr,
        _property: &Identifier,
        value: &Expr,
    ) -> Result<(), String> {
        object.accept(self)?;
        value.accept(self)?;
        Ok(())
    }

    fn visit_this(&mut self, ident: &Identifier) -> Result<(), String> {
        // now figure out if the target is a local or global var
        if let Some((depth, (slot, _))) = self.resolve_local(ident.name_str()) {
            // Store the resolved metadata back into the AST node if it was a local var.
            ident.swap_depth(depth);
            ident.swap_slot(slot);
        } else {
            return Err(format!(
                "'this' cannot be used in the global scope {}",
                ident.position()
            ));
        }
        Ok(())
    }
}
