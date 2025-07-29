use super::error::ConversionError;
use crate::lang::tokenizer::token::{Token, TokenType};
use crate::lang::view::View;
use crate::lang::visitor::Visitor;
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
            Self::Equal { view: _ } => write!(f, "'=='"),
            Self::NotEqual { view: _ } => write!(f, "'!='"),
            Self::Less { view: _ } => write!(f, "'<'"),
            Self::LessEqual { view: _ } => write!(f, "'<='"),
            Self::Greater { view: _ } => write!(f, "'>'"),
            Self::GreaterEqual { view: _ } => write!(f, "'>='"),
            Self::Plus { view: _ } => write!(f, "'+'"),
            Self::Minus { view: _ } => write!(f, "'-'"),
            Self::Star { view: _ } => write!(f, "'*'"),
            Self::Slash { view: _ } => write!(f, "'/'"),
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
            Self::Bang { view: _ } => write!(f, "'!'"),
            Self::Minus { view: _ } => write!(f, "'-'"),
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
            Literal::Number { value, view: _ } => write!(f, "{}", value),
            Literal::String { value, view: _ } => write!(f, "\"{}\"", value),
            Literal::Boolean { value, view: _ } => write!(f, "{}", value),
            Literal::Nil { view: _ } => write!(f, "nil"),
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
}

impl Expr {
    pub fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Binary { left, op, right } => v.visit_binary(left, *op, right),
            Expr::Grouping { expr } => v.visit_grouping(expr),
            Expr::Literal { value } => v.visit_literal(value),
            Expr::Unary { prefix, value } => v.visit_unary(*prefix, value),
        }
    }
}
