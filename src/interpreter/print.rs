// use crate::lang::tree::ast::{BinaryOperator, Expr, Identifier, Literal, UnaryPrefix};
// use crate::lang::visitor::Visitor;

// pub struct PrintVisitor;

// impl PrintVisitor {
//     pub fn visit(&mut self, expr: Expr) {
//         expr.accept(self);
//         print!("\n");
//     }
// }

// impl Visitor<()> for PrintVisitor {
//     fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) {
//         left.accept(self);
//         print!("{}", op);
//         right.accept(self);
//     }
//     fn visit_grouping(&mut self, expr: &Expr) -> () {
//         print!("(");
//         expr.accept(self);
//         print!(")");
//     }
//     fn visit_literal(&mut self, value: &Literal) -> () {
//         print!("{}", value);
//     }
//     fn visit_unary(&mut self, prefix: UnaryPrefix, value: &Expr) -> () {
//         print!("{}", prefix);
//         value.accept(self);
//     }
//     // todo: if needed go do these one day
//     fn visit_expression_statement(&mut self, _: &Expr) -> () {}
//     fn visit_print_statement(&mut self, _: &Expr) -> () {}
//     fn visit_var_statement(&mut self, _: &Identifier, _: Option<&Expr>) -> () {}
//     fn visit_variable(&mut self, _: &Identifier) -> () {}
//     fn visit_assignment(&mut self, _: &Identifier, _: &Expr) -> () {}
//     fn visit_block_statement(&mut self, _: &[crate::lang::tree::ast::Stmt]) -> () {}
// }
