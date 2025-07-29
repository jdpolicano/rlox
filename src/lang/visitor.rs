use super::tree::ast::{BinaryOperator, Expr, Identifier, Literal, UnaryPrefix};

pub trait Visitor<T> {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_literal(&mut self, value: &Literal) -> T;
    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> T;
    fn visit_variable(&mut self, name: &Identifier) -> T;
    fn visit_expression_statement(&mut self, expr: &Expr) -> T;
    fn visit_print_statement(&mut self, expr: &Expr) -> T;
    fn visit_var_statement(&mut self, name: &Identifier, expr: Option<&Expr>) -> T;
}
