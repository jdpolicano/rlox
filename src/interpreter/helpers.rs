use crate::interpreter::runtime::error::{BinaryError, LoxError, RuntimeError};
use crate::interpreter::runtime::eval::Eval;
use crate::interpreter::runtime::object::LoxObject;
use crate::lang::tree::ast::{BinaryOperator, Identifier, PropertyName, UnaryPrefix};

pub fn unary_op(value: &LoxObject, op: UnaryPrefix) -> Result<LoxObject, BinaryError> {
    match op {
        UnaryPrefix::Bang { .. } => Ok(value.truthy().into()),
        UnaryPrefix::Minus { .. } => apply_math_op(value, &(-1.0).into(), |a, b| a * b),
    }
}

pub fn binary_op(
    l: &LoxObject,
    r: &LoxObject,
    op: BinaryOperator,
) -> Result<LoxObject, BinaryError> {
    match op {
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

pub fn concat_strings(l: &LoxObject, r: &LoxObject) -> Result<LoxObject, BinaryError> {
    match (l.as_string(), r.as_string()) {
        (Some(a), Some(b)) => Ok(LoxObject::from((a.as_str(), b.as_str()))),
        _ => Err(BinaryError::InvalidTypes),
    }
}

pub fn apply_math_op<F>(l: &LoxObject, r: &LoxObject, f: F) -> Result<LoxObject, BinaryError>
where
    F: FnOnce(f64, f64) -> f64,
{
    match (l.as_number(), r.as_number()) {
        (Some(a), Some(b)) => Ok(LoxObject::from(f(a, b))),
        (None, _) => Err(BinaryError::LeftSide),
        (_, None) => Err(BinaryError::RightSide),
    }
}

pub fn apply_comparison<F>(l: &LoxObject, r: &LoxObject, f: F) -> Result<LoxObject, BinaryError>
where
    F: FnOnce(f64, f64) -> bool,
{
    match (l.as_number(), r.as_number()) {
        (Some(a), Some(b)) => Ok(LoxObject::from(f(a, b))),
        (None, _) => Err(BinaryError::LeftSide),
        (_, None) => Err(BinaryError::RightSide),
    }
}

pub fn binary_op_error(
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
        _ => format!("cannot add '{}' + {}'", l.type_str(), r.type_str()),
    };

    RuntimeError::new(LoxError::TypeError(msg), op.span())
}

pub fn unary_prefix_error(l: &LoxObject, prefix: UnaryPrefix) -> RuntimeError {
    let msg = format!("invalid type {} for prefix {}", l.type_str(), prefix);
    RuntimeError::new(LoxError::TypeError(msg), prefix.span())
}

pub fn reference_error(ident: &Identifier) -> RuntimeError {
    let msg = format!("undeclared identifier '{}'", ident.name_str());
    RuntimeError::new(LoxError::ReferenceError(msg), ident.span())
}

pub fn ref_error_prop_access(ident: &PropertyName) -> RuntimeError {
    let msg = format!("undefined property '{}'", ident.name_str());
    RuntimeError::new(LoxError::ReferenceError(msg), ident.span())
}

pub fn ref_error_prop_not_obj(ident: &PropertyName, t: &str) -> RuntimeError {
    let msg = format!(
        "cannont access property '{}' of non object type '{}'",
        ident.name_str(),
        t
    );
    RuntimeError::new(LoxError::ReferenceError(msg), ident.span())
}

pub fn type_error(expected: &str, received: &str) -> LoxError {
    LoxError::TypeError(format!(
        "expected type '{}' but received {}",
        expected, received
    ))
}

pub fn unwrap_to_object(eval: Eval) -> Result<LoxObject, LoxError> {
    match eval {
        Eval::Object(obj) => Ok(obj),
        _ => Err(type_error("object", eval.type_str())),
    }
}
