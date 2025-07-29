use super::runtime::error::{BinaryError, LoxError};
use crate::interpreter::runtime::value::LoxValue;
use crate::lang::tree::ast::{BinaryOperator, Expr, Literal, UnaryPrefix};
use crate::lang::visitor::Visitor;

// todo: implement lox errors. Should they just be a type of runtime value or should we simply use a result?
type LoxResult = Result<LoxValue, LoxError>;

pub struct Lox;

impl Lox {
    pub fn new() -> Self {
        Self
    }

    pub fn visit(mut self, root: Expr) -> LoxResult {
        root.accept(&mut self)
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
}

fn unary_op(value: &LoxValue, op: UnaryPrefix) -> Result<LoxValue, BinaryError> {
    match op {
        UnaryPrefix::Bang { view: _ } => Ok(value.truthy().into()),
        UnaryPrefix::Minus { view: _ } => apply_math_op(value, &(-1.0).into(), |a, b| a * b),
    }
}

fn binary_op(l: &LoxValue, r: &LoxValue, op: BinaryOperator) -> Result<LoxValue, BinaryError> {
    match op {
        // addition is a special case where we need to handle string concatenation.
        BinaryOperator::Plus { view: _ } => {
            if l.is_number() && r.is_number() {
                apply_math_op(l, r, |a, b| a + b)
            } else {
                concat_strings(l, r)
            }
        }
        BinaryOperator::Minus { view: _ } => apply_math_op(l, r, |a, b| a - b),
        BinaryOperator::Slash { view: _ } => apply_math_op(l, r, |a, b| a / b),
        BinaryOperator::Greater { view: _ } => apply_comparison(l, r, |a, b| a > b),
        BinaryOperator::GreaterEqual { view: _ } => apply_comparison(l, r, |a, b| a >= b),
        BinaryOperator::Less { view: _ } => apply_comparison(l, r, |a, b| a < b),
        BinaryOperator::LessEqual { view: _ } => apply_comparison(l, r, |a, b| a <= b),
        BinaryOperator::Equal { view: _ } => Ok(LoxValue::from(l == r)),
        BinaryOperator::NotEqual { view: _ } => Ok(LoxValue::from(l != r)),
        _ => Err(BinaryError::InvalidOperator),
    }
}

fn concat_strings(l: &LoxValue, r: &LoxValue) -> Result<LoxValue, BinaryError> {
    let l_as_str = l.as_string();
    let r_as_str = r.as_string();
    match (l_as_str, r_as_str) {
        (Some(a), Some(b)) => Ok(LoxValue::from((a.as_str(), b.as_str()))),
        _ => Err(BinaryError::RightSide),
    }
}

fn apply_math_op<F>(l: &LoxValue, r: &LoxValue, f: F) -> Result<LoxValue, BinaryError>
where
    F: FnOnce(f64, f64) -> f64,
{
    let l_as_num = l.as_number();
    let r_as_num = r.as_number();
    match (l_as_num, r_as_num) {
        (Some(a), Some(b)) => Ok(LoxValue::from(f(a, b))),
        _ => {
            if !l_as_num.is_some() {
                Err(BinaryError::LeftSide)
            } else {
                Err(BinaryError::RightSide)
            }
        }
    }
}

fn apply_comparison<F>(l: &LoxValue, r: &LoxValue, f: F) -> Result<LoxValue, BinaryError>
where
    F: FnOnce(f64, f64) -> bool,
{
    let l_as_num = l.as_number();
    let r_as_num = r.as_number();
    match (l_as_num, r_as_num) {
        (Some(a), Some(b)) => Ok(LoxValue::from(f(a, b))),
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
    l: &LoxValue,
    r: &LoxValue,
    op: BinaryOperator,
    err_type: BinaryError,
) -> LoxError {
    let msg = match err_type {
        BinaryError::LeftSide => {
            format!(
                "lefthand side incorrect type \"{}\" for op {}",
                l.type_str(),
                op
            )
        }
        BinaryError::RightSide => {
            format!(
                "righthand side incorrect type \"{}\" for op {}",
                r.type_str(),
                op
            )
        }
        BinaryError::InvalidOperator => {
            format!("invalid binary operator {}", op)
        }
    };

    LoxError::TypeError {
        msg,
        view: op.view(),
    }
}

fn unary_prefix_error(l: &LoxValue, prefix: UnaryPrefix) -> LoxError {
    let msg = format!("invalid type {} for prefix {}", l.type_str(), prefix);
    LoxError::TypeError {
        msg,
        view: prefix.view(),
    }
}
