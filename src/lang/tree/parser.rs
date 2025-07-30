use super::ast::Expr;
use super::error::ParseError;
use crate::lang::tokenizer::scanner::Scanner;
use crate::lang::tokenizer::token::{Token, TokenType};
use crate::lang::tree::ast::{BinaryOperator, Identifier, Literal, Stmt};
use crate::lang::view::View;
use std::iter::{Iterator, Peekable};

struct TokenStream<'a> {
    tokens: Peekable<Scanner<'a>>,
}

impl<'a> TokenStream<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            tokens: Scanner::new(src).peekable(),
        }
    }

    fn next(&mut self) -> Result<Token<'a>, ParseError> {
        if let Some(result) = self.tokens.next() {
            return result.map_err(|e| e.into());
        }
        Err(ParseError::UnexpectedEof)
    }

    fn next_if<F>(&mut self, condition: F) -> Option<Token<'a>>
    where
        F: FnOnce(&Token<'a>) -> bool,
    {
        if let Some(result) = self.tokens.peek() {
            match result {
                Ok(t) if condition(t) => return Some(self.next().unwrap()),
                _ => return None,
            }
        }
        None
    }

    fn peek(&mut self) -> Option<Result<&Token<'a>, ParseError>> {
        self.tokens
            .peek()
            .map(|r| r.as_ref().map_err(|e| e.clone().into()))
    }

    fn peek_next_if<F>(&mut self, condition: F) -> Result<Option<&Token<'a>>, ParseError>
    where
        F: FnOnce(&Token<'a>) -> bool,
    {
        if let Some(t) = self.tokens.peek() {
            match t {
                Ok(toke) if condition(toke) => return Ok(Some(toke)),
                Ok(_) => return Ok(None),
                Err(e) => return Err(e.clone().into()),
            }
        }
        Err(ParseError::UnexpectedEof)
    }
}

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
    statements: Vec<Stmt>,
    errors: Vec<ParseError>,
    loop_cnt: i8,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(src),
            statements: Vec::with_capacity(1024),
            errors: Vec::with_capacity(1024),
            loop_cnt: 0,
        }
    }

    pub fn parse(&mut self) {
        while !self.is_done() {
            match self.declaration() {
                Ok(stmt) => self.statements.push(stmt),
                Err(e) => {
                    println!("{}", e);
                    self.errors.push(e);
                    self.recover();
                }
            }
        }
    }

    pub fn had_errors(&self) -> bool {
        self.errors.len() > 0
    }

    pub fn take_statements(self) -> Vec<Stmt> {
        self.statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.is_var() {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.expect(
            "var delcaration requires an identifier",
            TokenType::Identifier,
        )?;

        let initializer = if let Some(_) = self.match_one(TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.expect("unterminated var statement", TokenType::Semicolon)?;

        Ok(Stmt::Var {
            name: name.try_into()?,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.is_print() {
            self.print_statement()
        } else if self.is_block() {
            self.block_statement()
        } else if self.is_if() {
            self.if_statement()
        } else if self.is_while() {
            self.while_statement()
        } else if self.is_for() {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.enter_loop();
        self.expect("for statement left parens", TokenType::LeftParen)?;

        let intializer = if self.is_semicolon() {
            None
        } else if self.is_var() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.is_semicolon() {
            None
        } else {
            let expr = self.expression()?;
            self.expect("for statement semicolon", TokenType::Semicolon)?;
            Some(expr)
        };

        let increment = if self.is_semicolon() {
            None
        } else {
            Some(self.expression()?)
        };

        self.expect("for statement right parens", TokenType::RightParen)?;
        let body = self.statement()?;
        self.exit_loop();
        desugar_for_statement(intializer, condition, increment, body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.enter_loop();
        self.expect("while statement left parens", TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.expect("while statement right parens", TokenType::RightParen)?;
        let block = Box::new(self.statement()?);
        self.exit_loop();
        Ok(Stmt::While { condition, block })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect("if statement left parens", TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.expect("if statement right parens", TokenType::RightParen)?;

        let if_block = Box::new(self.statement()?);

        let else_block = if self.is_else() {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            if_block,
            else_block,
        })
    }

    fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let terminates = |t: &'_ Token<'_>| {
            t.token_type != TokenType::RightBrace && t.token_type != TokenType::Eof
        };
        let mut statements = Vec::new();
        while let Some(_) = self.tokens.peek_next_if(terminates)? {
            statements.push(self.declaration()?);
        }
        self.expect("unclosed block scope", TokenType::RightBrace)?;
        Ok(Stmt::Block { statements })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.expect("unterminated print statement", TokenType::Semicolon)?;
        Ok(Stmt::Print { expr })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.expect("unterminated expression statement", TokenType::Semicolon)?;
        Ok(Stmt::Expression { expr })
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logical_or()?;
        if let Some(eq) = self.match_equal() {
            let assign_value = self.assignment()?;
            return match expr {
                Expr::Variable { value: name } => Ok(Expr::Assignment {
                    name,
                    value: Box::new(assign_value),
                }),
                _ => Err(ParseError::UnexpectedAssignment {
                    type_str: expr.type_str().to_string(),
                    location: eq.pos,
                }),
            };
        }

        if let Some(eq) = self.match_op_equal() {
            let assign_value = self.assignment()?;
            return match expr {
                Expr::Variable { value: name } => desugar_op_assignment(name, eq, assign_value),
                _ => Err(ParseError::UnexpectedAssignment {
                    type_str: expr.type_str().to_string(),
                    location: eq.pos,
                }),
            };
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.logical_and()?;
        while let Some(or) = self.match_or() {
            let rhs = self.logical_and()?;
            lhs = Expr::Logical {
                left: Box::new(lhs),
                op: or.try_into()?,
                right: Box::new(rhs),
            }
        }
        return Ok(lhs);
    }

    fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.equality()?;
        while let Some(or) = self.match_and() {
            let rhs = self.equality()?;
            lhs = Expr::Logical {
                left: Box::new(lhs),
                op: or.try_into()?,
                right: Box::new(rhs),
            }
        }
        return Ok(lhs);
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_equality() {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op.try_into()?,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while let Some(op) = self.match_comparison() {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op.try_into()?,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while let Some(op) = self.match_term() {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op.try_into()?,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while let Some(op) = self.match_factor() {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: op.try_into()?,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(op) = self.match_unary() {
            Ok(Expr::Unary {
                prefix: op.try_into()?,
                value: Box::new(self.unary()?),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.is_left_paren() {
            let expr = self.expression()?;
            let _ = self.expect(
                "primary grouping did not terminate correctly",
                TokenType::RightParen,
            )?;
            return Ok(Expr::Grouping {
                expr: Box::new(expr),
            });
        }

        if let Some(t) = self.match_break() {
            if !self.is_in_loop() {
                return Err(ParseError::InvalidLoopKeyword {
                    type_str: t.lexeme.to_string(),
                    location: t.pos,
                });
            }
            return Ok(Expr::Break);
        }

        if let Some(t) = self.match_continue() {
            if !self.is_in_loop() {
                return Err(ParseError::InvalidLoopKeyword {
                    type_str: t.lexeme.to_string(),
                    location: t.pos,
                });
            }
            return Ok(Expr::Continue);
        }

        if let Some(name) = self.match_one(TokenType::Identifier) {
            return Ok(Expr::Variable {
                value: name.try_into()?,
            });
        }

        let next_tok = self.tokens.next()?;
        let value = next_tok.try_into()?;
        Ok(Expr::Literal { value })
    }

    fn match_one(&mut self, t: TokenType) -> Option<Token<'a>> {
        self.tokens.next_if(|toke| toke.token_type == t)
    }

    fn match_many(&mut self, ts: &[TokenType]) -> Option<Token<'a>> {
        for t in ts {
            match self.match_one(*t) {
                Some(t) => return Some(t),
                _ => {}
            }
        }
        None
    }

    fn match_equality(&mut self) -> Option<Token<'a>> {
        self.match_many(&[TokenType::BangEqual, TokenType::EqualEqual])
    }

    fn match_comparison(&mut self) -> Option<Token<'a>> {
        self.match_many(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ])
    }

    fn match_term(&mut self) -> Option<Token<'a>> {
        self.match_many(&[TokenType::Plus, TokenType::Minus])
    }

    fn match_factor(&mut self) -> Option<Token<'a>> {
        self.match_many(&[TokenType::Slash, TokenType::Star])
    }

    fn match_unary(&mut self) -> Option<Token<'a>> {
        self.match_many(&[TokenType::Bang, TokenType::Minus])
    }

    fn match_and(&mut self) -> Option<Token<'a>> {
        self.match_one(TokenType::And)
    }

    fn match_or(&mut self) -> Option<Token<'a>> {
        self.match_one(TokenType::Or)
    }

    fn match_equal(&mut self) -> Option<Token<'a>> {
        self.match_one(TokenType::Equal)
    }

    fn match_break(&mut self) -> Option<Token<'a>> {
        self.match_one(TokenType::Break)
    }

    fn match_continue(&mut self) -> Option<Token<'a>> {
        self.match_one(TokenType::Continue)
    }

    fn match_op_equal(&mut self) -> Option<Token<'a>> {
        self.match_many(&[
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
        ])
    }
    // the semantics for is_<x> statments is that it returns a bool and throws the token away.
    // If you need to check and keep the token, use a matcher.
    fn is_print(&mut self) -> bool {
        self.match_one(TokenType::Print).is_some()
    }

    fn is_block(&mut self) -> bool {
        self.match_one(TokenType::LeftBrace).is_some()
    }

    fn is_if(&mut self) -> bool {
        self.match_one(TokenType::If).is_some()
    }

    fn is_else(&mut self) -> bool {
        self.match_one(TokenType::Else).is_some()
    }

    fn is_while(&mut self) -> bool {
        self.match_one(TokenType::While).is_some()
    }

    fn is_for(&mut self) -> bool {
        self.match_one(TokenType::For).is_some()
    }

    fn is_semicolon(&mut self) -> bool {
        self.match_one(TokenType::Semicolon).is_some()
    }

    fn is_var(&mut self) -> bool {
        self.match_one(TokenType::Var).is_some()
    }

    fn is_left_paren(&mut self) -> bool {
        self.match_one(TokenType::LeftParen).is_some()
    }

    fn expect(&mut self, msg: &'static str, t: TokenType) -> Result<Token<'a>, ParseError> {
        let toke = self.tokens.next()?;
        if toke.token_type != t {
            Err(ParseError::UnexpectedToken {
                expected: t,
                recieved: toke.to_string(),
                msg,
            })
        } else {
            Ok(toke)
        }
    }

    fn is_done(&mut self) -> bool {
        if let Some(result) = self.tokens.peek() {
            match result {
                Ok(t) if t.token_type == TokenType::Eof => return true,
                _ => return false,
            }
        }
        true
    }

    fn is_in_loop(&self) -> bool {
        self.loop_cnt > 0
    }

    fn enter_loop(&mut self) {
        self.loop_cnt += 1;
    }

    fn exit_loop(&mut self) {
        self.loop_cnt -= 1;
    }

    /// recover from a panic state by reading through until we hit the end of the stream, or alternatively a semi-colon terminator.
    fn recover(&mut self) {
        while let Some(result) = self.tokens.peek() {
            match result {
                Ok(toke) if toke.token_type == TokenType::Semicolon => {
                    let _ = self.tokens.next();
                    break;
                }
                Ok(toke) if toke.token_type == TokenType::Eof => {
                    break;
                }
                _ => {
                    let _ = self.tokens.next();
                }
            }
        }
    }
}

fn desugar_op_assignment(name: Identifier, op: Token<'_>, rhs: Expr) -> Result<Expr, ParseError> {
    let view = op.pos;
    let op = match op.token_type {
        TokenType::PlusEqual => BinaryOperator::Plus { view },
        TokenType::MinusEqual => BinaryOperator::Minus { view },
        TokenType::StarEqual => BinaryOperator::Star { view },
        TokenType::SlashEqual => BinaryOperator::Slash { view },
        _ => unreachable!("desugar should already be confirmed to be of a discrete set."),
    };
    Ok(Expr::Assignment {
        name: name.clone(),
        value: Box::new(Expr::Binary {
            left: Box::new(Expr::Variable { value: name }),
            op: op,
            right: Box::new(rhs),
        }),
    })
}

fn desugar_for_statement(
    initializer: Option<Stmt>,
    condition: Option<Expr>,
    increment: Option<Expr>,
    body: Stmt,
) -> Result<Stmt, ParseError> {
    let mut inner_block = vec![body];
    if let Some(inc) = increment {
        inner_block.push(make_expression_statment(inc))
    }
    let mut outer_block = vec![];
    if let Some(init) = initializer {
        outer_block.push(init);
    }
    let cond = condition.unwrap_or(make_true_expression());
    let while_stmt = make_while_statement(cond, inner_block);
    outer_block.push(while_stmt);
    Ok(Stmt::Block {
        statements: outer_block,
    })
}

fn make_expression_statment(expr: Expr) -> Stmt {
    Stmt::Expression { expr }
}

fn make_while_statement(condition: Expr, stmts: Vec<Stmt>) -> Stmt {
    Stmt::While {
        condition,
        block: Box::new(make_block_statement(stmts)),
    }
}

fn make_block_statement(stmts: Vec<Stmt>) -> Stmt {
    Stmt::Block { statements: stmts }
}

fn make_true_expression() -> Expr {
    // it is okay to make up the "view" here because it is synthetic and can never fail at runtime reasonably.
    Expr::Literal {
        value: Literal::Boolean {
            value: true,
            view: View::default(),
        },
    }
}
