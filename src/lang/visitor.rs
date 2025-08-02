use super::tree::ast::{
    BinaryOperator, Callee, Expr, Identifier, Literal, LogicalOperator, Stmt, UnaryPrefix,
};
use std::rc::Rc;

pub trait Visitor<T> {
    // expressions
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> T;
    fn visit_logical(&mut self, left: &Expr, op: LogicalOperator, right: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_literal(&mut self, value: &Literal) -> T;
    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> T;
    fn visit_variable(&mut self, name: &Identifier) -> T;
    fn visit_assignment(&mut self, name: &Identifier, value: &Expr) -> T;
    fn visit_call(&mut self, callee: &Callee, args: &[Expr]) -> T;
    // statments
    fn visit_expression_statement(&mut self, expr: &Expr) -> T;
    fn visit_print_statement(&mut self, expr: &Expr) -> T;
    fn visit_var_statement(&mut self, name: &Identifier, expr: Option<&Expr>) -> T;
    fn visit_block_statement(&mut self, statments: &[Stmt]) -> T;
    fn visit_if_statement(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: Option<&Stmt>,
    ) -> T;
    fn visit_while_statement(&mut self, condition: &Expr, block: &Stmt) -> T;
    fn visit_function_statement(
        &mut self,
        name: &Identifier,
        params: &[Identifier],
        body: Rc<Stmt>,
    ) -> T;
    fn visit_break_statement(&mut self) -> T;
    fn visit_continue_statment(&mut self) -> T;
    fn visit_return_statment(&mut self, value: Option<&Expr>) -> T;
}
