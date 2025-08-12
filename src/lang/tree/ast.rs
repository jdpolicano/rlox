use super::error::ConversionError;
use crate::lang::tokenizer::span::Span;
use crate::lang::tokenizer::token::{Token, TokenType};
use crate::lang::visitor::Visitor;
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
// "==" | "!=" | "<" | "<=" | ">" | ">=" |
// "+"  | "-"  | "*" | "/" ;
#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Equal(Span),
    NotEqual(Span),
    Less(Span),
    LessEqual(Span),
    Greater(Span),
    GreaterEqual(Span),
    Plus(Span),
    Minus(Span),
    Star(Span),
    Slash(Span),
}

impl TryFrom<Token<'_>> for BinaryOperator {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::EqualEqual => Ok(BinaryOperator::Equal(value.span)),
            TokenType::BangEqual => Ok(BinaryOperator::NotEqual(value.span)),
            TokenType::Less => Ok(BinaryOperator::Less(value.span)),
            TokenType::LessEqual => Ok(BinaryOperator::LessEqual(value.span)),
            TokenType::Greater => Ok(BinaryOperator::Greater(value.span)),
            TokenType::GreaterEqual => Ok(BinaryOperator::GreaterEqual(value.span)),
            TokenType::Plus => Ok(BinaryOperator::Plus(value.span)),
            TokenType::Minus => Ok(BinaryOperator::Minus(value.span)),
            TokenType::Star => Ok(BinaryOperator::Star(value.span)),
            TokenType::Slash => Ok(BinaryOperator::Slash(value.span)),
            _ => {
                return Err(ConversionError::InvalidBinaryOperator(value.into()));
            }
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Equal(_) => write!(f, "='"),
            Self::NotEqual(_) => write!(f, "!'"),
            Self::Less(_) => write!(f, "<"),
            Self::LessEqual(_) => write!(f, "<'"),
            Self::Greater(_) => write!(f, ">"),
            Self::GreaterEqual(_) => write!(f, ">'"),
            Self::Plus(_) => write!(f, "+"),
            Self::Minus(_) => write!(f, "-"),
            Self::Star(_) => write!(f, "*"),
            Self::Slash(_) => write!(f, "/"),
        }
    }
}

impl BinaryOperator {
    pub fn span(&self) -> Span {
        match self {
            Self::Equal(span) => *span,
            Self::NotEqual(span) => *span,
            Self::Less(span) => *span,
            Self::LessEqual(span) => *span,
            Self::Greater(span) => *span,
            Self::GreaterEqual(span) => *span,
            Self::Plus(span) => *span,
            Self::Minus(span) => *span,
            Self::Star(span) => *span,
            Self::Slash(span) => *span,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And(Span),
    Or(Span),
}

impl TryFrom<Token<'_>> for LogicalOperator {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::And => Ok(LogicalOperator::And(value.span)),
            TokenType::Or => Ok(LogicalOperator::Or(value.span)),
            _ => {
                return Err(ConversionError::InvalidLogicalOperator(value.into()));
            }
        }
    }
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::And(_) => write!(f, "and"),
            Self::Or(_) => write!(f, "or"),
        }
    }
}

impl LogicalOperator {
    pub fn span(&self) -> Span {
        match self {
            Self::And(span) => *span,
            Self::Or(span) => *span,
        }
    }
}

//
// "!" | "-" prefix
#[derive(Debug, Clone, Copy)]
pub enum UnaryPrefix {
    Bang(Span),
    Minus(Span),
}

impl TryFrom<Token<'_>> for UnaryPrefix {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::Bang => Ok(UnaryPrefix::Bang(value.span)),
            TokenType::Minus => Ok(UnaryPrefix::Minus(value.span)),
            _ => {
                return Err(ConversionError::InvalidUnaryOperator(value.into()));
            }
        }
    }
}

impl fmt::Display for UnaryPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bang(_) => write!(f, "!"),
            Self::Minus(_) => write!(f, "-"),
        }
    }
}

impl UnaryPrefix {
    pub fn span(&self) -> Span {
        match self {
            UnaryPrefix::Bang(span) => *span,
            UnaryPrefix::Minus(span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number { value: f64, span: Span },
    String { value: Rc<String>, span: Span },
    Boolean { value: bool, span: Span },
    Nil { span: Span },
}

impl Literal {
    pub fn new_number(n: f64, span: Span) -> Self {
        Self::Number { value: n, span }
    }

    pub fn new_string(s: String, span: Span) -> Self {
        Self::String {
            value: Rc::new(s),
            span,
        }
    }

    pub fn new_boolean(b: bool, span: Span) -> Self {
        Self::Boolean { value: b, span }
    }

    pub fn new_nil(span: Span) -> Self {
        Self::Nil { span }
    }
}

impl TryFrom<Token<'_>> for Literal {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::Number => {
                let num = value.lexeme.parse::<f64>();
                if num.is_err() {
                    Err(ConversionError::InvalidNumber(value.into()))
                } else {
                    Ok(Literal::new_number(num.unwrap(), value.span))
                }
            }
            TokenType::String => {
                let end = value.lexeme.len() - 1;
                Ok(Literal::new_string(
                    value.lexeme[1..end].to_string(),
                    value.span,
                ))
            }
            TokenType::True => Ok(Literal::new_boolean(true, value.span)),
            TokenType::False => Ok(Literal::new_boolean(false, value.span)),
            TokenType::Nil => Ok(Literal::new_nil(value.span)),
            _ => {
                return Err(ConversionError::InvalidLiteralType(value.into()));
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Number { value, .. } => write!(f, "{}", value),
            Literal::String { value, .. } => write!(f, "{}", value),
            Literal::Boolean { value, .. } => write!(f, "{}", value),
            Literal::Nil { .. } => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    Global,
    Local { slot: usize, depth: usize },
    UpValue { index: usize },
}

#[derive(Debug, Clone)]
pub struct Identifier {
    name: String,
    binding: Cell<Option<Binding>>,
    span: Span,
}

impl Identifier {
    pub fn name_str(&self) -> &str {
        self.name.as_str()
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn set_local_binding(&self, depth: usize, slot: usize) {
        self.binding.replace(Some(Binding::Local { slot, depth }));
    }

    pub fn set_global_binding(&self) {
        self.binding.replace(Some(Binding::Global));
    }

    pub fn set_upvalue_binding(&self, index: usize) {
        self.binding.replace(Some(Binding::UpValue { index }));
    }

    pub fn is_global(&self) -> bool {
        if let Some(binding) = self.binding.get() {
            return binding == Binding::Global;
        }
        false
    }

    pub fn depth_slot(&self) -> Option<(usize, usize)> {
        if let Some(Binding::Local { depth, slot }) = self.binding.get() {
            return Some((depth, slot));
        }
        None
    }

    pub fn upvalue(&self) -> Option<usize> {
        if let Some(Binding::UpValue { index }) = self.binding.get() {
            return Some(index);
        }
        None
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<Token<'_>> for Identifier {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            // you can convert a fun to an identifier because
            // we support anonymous functions whose name essentially becomes the
            // location where it was declared.
            TokenType::Identifier | TokenType::Fun | TokenType::This => Ok(Self {
                name: value.lexeme.to_string(),
                span: value.span,
                binding: Cell::new(None),
            }),
            _ => Err(ConversionError::InvalidIdentifier(value.into())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyName {
    name: String,
    span: Span,
}

impl PropertyName {
    pub fn name_str(&self) -> &str {
        self.name.as_str()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for PropertyName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<Token<'_>> for PropertyName {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            // you can convert a fun to an identifier because
            // we support anonymous functions whose name essentially becomes the
            // location where it was declared.
            TokenType::Identifier => Ok(Self {
                name: value.lexeme.to_string(),
                span: value.span,
            }),
            _ => Err(ConversionError::InvalidIdentifier(value.into())),
        }
    }
}

#[derive(Debug)]
pub struct Callee {
    pub expr: Box<Expr>,
    span: Span,
}

impl Callee {
    pub fn new(expr: Expr, span: Span) -> Self {
        Self {
            expr: Box::new(expr),
            span,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug)]
pub struct Function {
    name: Option<Identifier>,
    params: Vec<Identifier>,
    body: Rc<Stmt>,
    // marker position is the fallback location we'll point out
    // if we encounter an issue with this function.
    // The default is the name of the function if its available, this helps
    // handle anonymous functions.
    span: Span,
    // this tells us whether or not the function is a static function, declared on the class instance itself.
    is_static: bool,
}

impl Function {
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn span(&self) -> Span {
        self.name
            .as_ref()
            .map(|ident| ident.span())
            .unwrap_or(self.span)
    }

    pub fn is_anonymous(&self) -> bool {
        self.name.is_none()
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }

    pub fn params(&self) -> &[Identifier] {
        &self.params[..]
    }

    pub fn param_strings(&self) -> Vec<String> {
        self.params()
            .iter()
            .map(|p| p.name_str().to_string())
            .collect()
    }

    pub fn body(&self) -> Rc<Stmt> {
        self.body.clone()
    }

    pub fn name(&self) -> Option<Identifier> {
        self.name.clone()
    }

    pub fn new(
        name: Option<Identifier>,
        params: Vec<Identifier>,
        body: Rc<Stmt>,
        span: Span,
        is_static: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            span,
            is_static,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
        span: Span,
    },

    Logical {
        left: Box<Expr>,
        op: LogicalOperator,
        right: Box<Expr>,
        span: Span,
    },

    Grouping {
        expr: Box<Expr>,
        span: Span,
    },

    Literal {
        value: Literal,
        span: Span,
    },

    Unary {
        prefix: UnaryPrefix,
        value: Box<Expr>,
        span: Span,
    },

    Variable {
        value: Identifier,
        span: Span,
    },

    Assignment {
        name: Identifier,
        value: Box<Expr>,
        span: Span,
    },

    Call {
        callee: Callee,
        args: Vec<Expr>,
        span: Span,
    },

    Function {
        value: Function,
        span: Span,
    },

    Get {
        object: Box<Expr>,
        property: PropertyName,
        span: Span,
    },

    Set {
        object: Box<Expr>,
        property: PropertyName,
        value: Box<Expr>,
        span: Span,
    },

    This {
        // it needs to be an identifier because we will look it up like any other variable name.
        ident: Identifier,
        span: Span,
    },
}

impl Expr {
    pub fn accept<T, V>(&self, v: &mut V) -> T
    where
        V: Visitor<T, Expr, Stmt>,
    {
        match self {
            Expr::Binary {
                left, op, right, ..
            } => v.visit_binary(left, *op, right),
            Expr::Grouping { expr, .. } => v.visit_grouping(expr),
            Expr::Literal { value, .. } => v.visit_literal(value),
            Expr::Unary { prefix, value, .. } => v.visit_unary(*prefix, value),
            Expr::Variable { value, .. } => v.visit_variable(value),
            Expr::Assignment { name, value, .. } => v.visit_assignment(name, value),
            Expr::Logical {
                left, op, right, ..
            } => v.visit_logical(left, *op, right),
            Expr::Call { callee, args, .. } => v.visit_call(callee, args),
            Expr::Function { value, .. } => v.visit_function(value),
            Expr::Get {
                object, property, ..
            } => v.visit_get(object, property),
            Expr::Set {
                object,
                property,
                value,
                ..
            } => v.visit_set(object, property, value),
            Expr::This { ident, .. } => v.visit_this(ident),
        }
    }

    pub fn type_str(&self) -> &str {
        match self {
            Self::Binary { .. } => "binary",
            Self::Grouping { .. } => "grouping",
            Self::Literal { .. } => "literal",
            Self::Unary { .. } => "unary",
            Self::Variable { .. } => "var",
            Self::Assignment { .. } => "assignment",
            Self::Logical { .. } => "logical",
            Self::Call { .. } => "call",
            Self::Function { .. } => "function expression",
            Self::Get { .. } => "get",
            Self::Set { .. } => "set",
            Self::This { .. } => "this",
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Binary { span, .. } => *span,
            Self::Grouping { span, .. } => *span,
            Self::Literal { span, .. } => *span,
            Self::Unary { span, .. } => *span,
            Self::Variable { span, .. } => *span,
            Self::Assignment { span, .. } => *span,
            Self::Logical { span, .. } => *span,
            Self::Call { span, .. } => *span,
            Self::Function { span, .. } => *span,
            Self::Get { span, .. } => *span,
            Self::Set { span, .. } => *span,
            Self::This { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expr: Expr,
        span: Span,
    },

    Print {
        expr: Expr,
        span: Span,
    },

    Var {
        name: Identifier,
        initializer: Option<Expr>,
        span: Span,
    },

    Block {
        statements: Vec<Stmt>,
        span: Span,
    },

    If {
        condition: Expr,
        if_block: Box<Stmt>,
        else_block: Option<Box<Stmt>>,
        span: Span,
    },

    While {
        condition: Expr,
        block: Box<Stmt>,
        span: Span,
    },

    Class {
        name: Identifier,
        super_class: Option<Expr>,
        methods: Vec<Function>,
        span: Span,
    },

    Break(Span),
    Continue(Span),
    Return {
        value: Option<Expr>,
        span: Span,
    },
}

impl Stmt {
    pub fn accept<T, V>(&self, v: &mut V) -> T
    where
        V: Visitor<T, Expr, Stmt>,
    {
        match self {
            Self::Expression { expr, .. } => v.visit_expression_statement(expr),
            Self::Print { expr, .. } => v.visit_print_statement(expr),
            Self::Var {
                name, initializer, ..
            } => v.visit_var_statement(name, initializer.as_ref()),
            Self::Block { statements, .. } => v.visit_block_statement(statements),
            Self::If {
                condition,
                if_block,
                else_block,
                ..
            } => v.visit_if_statement(
                condition,
                if_block,
                else_block.as_ref().map(|stmt| stmt.as_ref()),
            ),
            Self::While {
                condition, block, ..
            } => v.visit_while_statement(condition, block),

            Self::Break(_) => v.visit_break_statement(),
            Self::Continue(_) => v.visit_continue_statment(),
            Self::Return { value, .. } => v.visit_return_statment(value.as_ref()),
            Self::Class {
                name,
                super_class,
                methods,
                ..
            } => v.visit_class_statement(name, super_class.as_ref(), methods),
        }
    }

    pub fn type_str(&self) -> &str {
        match self {
            Stmt::Expression { .. } => "expression",
            Stmt::Print { .. } => "print",
            Stmt::Var { .. } => "var",
            Stmt::Block { .. } => "block",
            Self::If { .. } => "if",
            Self::While { .. } => "while",
            Self::Break(_) => "break",
            Self::Continue(_) => "continue",
            Self::Return { .. } => "return",
            Self::Class { .. } => "class",
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Stmt::Expression { span, .. } => *span,
            Stmt::Print { span, .. } => *span,
            Stmt::Var { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Self::If { span, .. } => *span,
            Self::While { span, .. } => *span,
            Self::Break(span) => *span,
            Self::Continue(span) => *span,
            Self::Return { span, .. } => *span,
            Self::Class { span, .. } => *span,
        }
    }
}
