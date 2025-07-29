use super::tree::ast::{BinaryOperator, Expr, Literal, UnaryPrefix};

pub trait Visitor<T> {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> T;
    fn visit_grouping(&mut self, expr: &Expr) -> T;
    fn visit_literal(&mut self, value: &Literal) -> T;
    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> T;
}
