use super::error::ConversionError;
use crate::lang::tokenizer::token::{Token, TokenType};
use crate::lang::view::View;
use crate::lang::visitor::Visitor;
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
// "==" | "!=" | "<" | "<=" | ">" | ">=" |
// "+"  | "-"  | "*" | "/" ;
#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Equal { view: View },
    NotEqual { view: View },
    Less { view: View },
    LessEqual { view: View },
    Greater { view: View },
    GreaterEqual { view: View },
    Plus { view: View },
    Minus { view: View },
    Star { view: View },
    Slash { view: View },
}

impl TryFrom<Token<'_>> for BinaryOperator {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::EqualEqual => Ok(BinaryOperator::Equal { view: value.pos }),
            TokenType::BangEqual => Ok(BinaryOperator::NotEqual { view: value.pos }),
            TokenType::Less => Ok(BinaryOperator::Less { view: value.pos }),
            TokenType::LessEqual => Ok(BinaryOperator::LessEqual { view: value.pos }),
            TokenType::Greater => Ok(BinaryOperator::Greater { view: value.pos }),
            TokenType::GreaterEqual => Ok(BinaryOperator::GreaterEqual { view: value.pos }),
            TokenType::Plus => Ok(BinaryOperator::Plus { view: value.pos }),
            TokenType::Minus => Ok(BinaryOperator::Minus { view: value.pos }),
            TokenType::Star => Ok(BinaryOperator::Star { view: value.pos }),
            TokenType::Slash => Ok(BinaryOperator::Slash { view: value.pos }),
            _ => {
                return Err(ConversionError::InvalidBinaryOperator(value.into()));
            }
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Equal { .. } => write!(f, "'=='"),
            Self::NotEqual { .. } => write!(f, "'!='"),
            Self::Less { .. } => write!(f, "'<'"),
            Self::LessEqual { .. } => write!(f, "'<='"),
            Self::Greater { .. } => write!(f, "'>'"),
            Self::GreaterEqual { .. } => write!(f, "'>='"),
            Self::Plus { .. } => write!(f, "'+'"),
            Self::Minus { .. } => write!(f, "'-'"),
            Self::Star { .. } => write!(f, "'*'"),
            Self::Slash { .. } => write!(f, "'/'"),
        }
    }
}

impl BinaryOperator {
    pub fn view(&self) -> View {
        match self {
            Self::Equal { view } => *view,
            Self::NotEqual { view } => *view,
            Self::Less { view } => *view,
            Self::LessEqual { view } => *view,
            Self::Greater { view } => *view,
            Self::GreaterEqual { view } => *view,
            Self::Plus { view } => *view,
            Self::Minus { view } => *view,
            Self::Star { view } => *view,
            Self::Slash { view } => *view,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And { view: View },
    Or { view: View },
}

impl TryFrom<Token<'_>> for LogicalOperator {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::And => Ok(LogicalOperator::And { view: value.pos }),
            TokenType::Or => Ok(LogicalOperator::Or { view: value.pos }),
            _ => {
                return Err(ConversionError::InvalidLogicalOperator(value.into()));
            }
        }
    }
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::And { .. } => write!(f, "and'"),
            Self::Or { .. } => write!(f, "'or'"),
        }
    }
}

impl LogicalOperator {
    pub fn view(&self) -> View {
        match self {
            Self::And { view } => *view,
            Self::Or { view } => *view,
        }
    }
}

// "!" | "-" prefix
#[derive(Debug, Clone, Copy)]
pub enum UnaryPrefix {
    Bang { view: View },
    Minus { view: View },
}

impl TryFrom<Token<'_>> for UnaryPrefix {
    type Error = ConversionError;
    fn try_from(value: Token<'_>) -> Result<Self, Self::Error> {
        match value.token_type {
            TokenType::Bang => Ok(UnaryPrefix::Bang { view: value.pos }),
            TokenType::Minus => Ok(UnaryPrefix::Minus { view: value.pos }),
            _ => {
                return Err(ConversionError::InvalidUnaryOperator(value.into()));
            }
        }
    }
}

impl fmt::Display for UnaryPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bang { .. } => write!(f, "'!'"),
            Self::Minus { .. } => write!(f, "'-'"),
        }
    }
}

impl UnaryPrefix {
    pub fn view(&self) -> View {
        match self {
            UnaryPrefix::Bang { view } => *view,
            UnaryPrefix::Minus { view } => *view,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number { value: f64, view: View },
    String { value: Rc<String>, view: View },
    Boolean { value: bool, view: View },
    Nil { view: View },
}

impl Literal {
    pub fn new_number(n: f64, v: View) -> Self {
        Self::Number { value: n, view: v }
    }

    pub fn new_string(s: String, v: View) -> Self {
        Self::String {
            value: Rc::new(s),
            view: v,
        }
    }

    pub fn new_boolean(b: bool, v: View) -> Self {
        Self::Boolean { value: b, view: v }
    }

    pub fn new_nil(v: View) -> Self {
        Self::Nil { view: v }
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
                    Ok(Literal::new_number(num.unwrap(), value.pos))
                }
            }
            TokenType::String => {
                let end = value.lexeme.len() - 1;
                Ok(Literal::new_string(
                    value.lexeme[1..end].to_string(),
                    value.pos,
                ))
            }
            TokenType::True => Ok(Literal::new_boolean(true, value.pos)),
            TokenType::False => Ok(Literal::new_boolean(false, value.pos)),
            TokenType::Nil => Ok(Literal::new_nil(value.pos)),
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
            Literal::String { value, .. } => write!(f, "\"{}\"", value),
            Literal::Boolean { value, .. } => write!(f, "{}", value),
            Literal::Nil { .. } => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    name: String,
    slot: Cell<Option<usize>>,
    depth: Cell<Option<usize>>,
    view: View,
}

impl Identifier {
    pub fn name_str(&self) -> &str {
        self.name.as_str()
    }

    pub fn view(&self) -> View {
        self.view
    }

    pub fn swap_depth(&self, value: usize) {
        self.depth.replace(Some(value));
    }

    pub fn swap_slot(&self, value: usize) {
        self.slot.replace(Some(value));
    }

    pub fn is_global(&self) -> bool {
        self.slot.get().is_none() || self.depth.get().is_none()
    }

    pub fn depth_slot(&self) -> Option<(usize, usize)> {
        // if self.name_str() == "count" {
        //     println!("printing self to get depth slot -> {:#?}", self);
        // }
        if let Some(depth) = self.depth.get() {
            if let Some(slot) = self.slot.get() {
                return Some((depth, slot));
            }
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
                view: value.pos,
                slot: Cell::new(None),
                depth: Cell::new(None),
            }),
            _ => Err(ConversionError::InvalidIdentifier(value.into())),
        }
    }
}

#[derive(Debug)]
pub struct Callee {
    pub expr: Box<Expr>,
    pub view: View,
}

impl Callee {
    pub fn view(&self) -> View {
        self.view
    }
}

#[derive(Debug)]
pub struct Function {
    name: Option<Identifier>,
    params: Vec<Identifier>,
    body: Rc<Stmt>,
    // marker view is the fallback location we'll point out
    // if we encounter an issue with this function.
    // The default is the name of the function if its available.
    marker_view: View,
}

impl Function {
    pub fn with_view(mut self, v: View) -> Self {
        self.marker_view = v;
        self
    }

    pub fn view(&self) -> View {
        self.name
            .as_ref()
            .map(|ident| ident.view())
            .unwrap_or(self.marker_view)
    }

    pub fn is_anonymous(&self) -> bool {
        self.name.is_none()
    }

    pub fn params(&self) -> &[Identifier] {
        &self.params[..]
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
        marker_view: View,
    ) -> Self {
        Self {
            name,
            params,
            body,
            marker_view,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },

    Logical {
        left: Box<Expr>,
        op: LogicalOperator,
        right: Box<Expr>,
    },

    Grouping {
        expr: Box<Expr>,
    },

    Literal {
        value: Literal,
    },

    Unary {
        prefix: UnaryPrefix,
        value: Box<Expr>,
    },

    Variable {
        value: Identifier,
    },

    Assignment {
        name: Identifier,
        value: Box<Expr>,
    },

    Call {
        callee: Callee,
        args: Vec<Expr>,
    },

    Function {
        value: Function,
    },

    Get {
        object: Box<Expr>,
        property: Identifier,
    },

    Set {
        object: Box<Expr>,
        property: Identifier,
        value: Box<Expr>,
    },

    This {
        // it needs to be an identifier because we will look it up like any other variable name.
        ident: Identifier,
    },
}

impl Expr {
    pub fn accept<T, V>(&self, v: &mut V) -> T
    where
        V: Visitor<T, Expr, Stmt>,
    {
        match self {
            Expr::Binary { left, op, right } => v.visit_binary(left, *op, right),
            Expr::Grouping { expr } => v.visit_grouping(expr),
            Expr::Literal { value } => v.visit_literal(value),
            Expr::Unary { prefix, value } => v.visit_unary(*prefix, value),
            Expr::Variable { value } => v.visit_variable(value),
            Expr::Assignment { name, value } => v.visit_assignment(name, value),
            Expr::Logical { left, op, right } => v.visit_logical(left, *op, right),
            Expr::Call { callee, args } => v.visit_call(callee, args),
            Expr::Function { value } => v.visit_function(value),
            Expr::Get { object, property } => v.visit_get(object, property),
            Expr::Set {
                object,
                property,
                value,
            } => v.visit_set(object, property, value),
            Expr::This { ident } => v.visit_this(ident),
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
}

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expr: Expr,
    },

    Print {
        expr: Expr,
    },

    Var {
        name: Identifier,
        initializer: Option<Expr>,
    },

    Block {
        statements: Vec<Stmt>,
    },

    If {
        condition: Expr,
        if_block: Box<Stmt>,
        else_block: Option<Box<Stmt>>,
    },

    While {
        condition: Expr,
        block: Box<Stmt>,
    },

    Class {
        name: Identifier,
        methods: Vec<Function>,
    },

    Break,
    Continue,
    Return {
        value: Option<Expr>,
    },
}

impl Stmt {
    pub fn accept<T, V>(&self, v: &mut V) -> T
    where
        V: Visitor<T, Expr, Stmt>,
    {
        match self {
            Self::Expression { expr } => v.visit_expression_statement(expr),
            Self::Print { expr } => v.visit_print_statement(expr),
            Self::Var { name, initializer } => v.visit_var_statement(name, initializer.as_ref()),
            Self::Block { statements } => v.visit_block_statement(statements),
            Self::If {
                condition,
                if_block,
                else_block,
            } => v.visit_if_statement(
                condition,
                if_block,
                else_block.as_ref().map(|stmt| stmt.as_ref()),
            ),
            Self::While { condition, block } => v.visit_while_statement(condition, block),

            Self::Break => v.visit_break_statement(),
            Self::Continue => v.visit_continue_statment(),
            Self::Return { value } => v.visit_return_statment(value.as_ref()),
            Self::Class { name, methods } => v.visit_class_statement(name, methods),
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
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Return { .. } => "return",
            Self::Class { .. } => "class",
        }
    }
}
