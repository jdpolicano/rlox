use super::ast::Expr;
use super::error::ParseError;
use crate::lang::tokenizer::scanner::Scanner;
use crate::lang::tokenizer::token::{Token, TokenType};
use crate::lang::tree::ast::{BinaryOperator, Callee, Function, Identifier, Literal, Stmt};
use crate::lang::view::View;
use std::iter::{Iterator, Peekable};
use std::rc::Rc;

const MAX_FUNC_ARGS: usize = 255;

struct TokenStream<'a> {
    tokens: Peekable<Scanner<'a>>,
    last_token: Option<Token<'a>>,
}

impl<'a> TokenStream<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            tokens: Scanner::new(src).peekable(),
            last_token: None,
        }
    }

    fn next(&mut self) -> Result<Token<'a>, ParseError> {
        if let Some(result) = self.tokens.next() {
            let token = result.map_err(|e| ParseError::from(e))?;
            self.last_token = Some(token.clone());
            return Ok(token);
        }
        Err(ParseError::UnexpectedEof)
    }

    fn next_if<F>(&mut self, condition: F) -> Option<Token<'a>>
    where
        F: FnOnce(&Token<'a>) -> bool,
    {
        if let Some(result) = self.tokens.peek() {
            match result {
                Ok(t) if condition(t) => {
                    let token = self.next().unwrap();
                    return Some(token);
                }
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

    fn assert(&mut self, t: TokenType, msg: &'static str) -> Result<Token<'a>, ParseError> {
        let token = self.next()?;
        if token.token_type != t {
            return Err(ParseError::UnexpectedToken {
                expected: t,
                recieved: token.token_type.to_string(),
                msg,
            });
        }
        Ok(token)
    }

    fn last(&self) -> Option<&Token<'a>> {
        self.last_token.as_ref()
    }
}

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
    statements: Vec<Stmt>,
    errors: Vec<ParseError>,
    loop_cnt: i8,
    fn_cnt: i8,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(src),
            statements: Vec::with_capacity(1024),
            errors: Vec::with_capacity(1024),
            loop_cnt: 0,
            fn_cnt: 0,
        }
    }

    pub fn parse(&mut self) {
        while !self.take_done() {
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
        if self.match_one(TokenType::Var).is_some() {
            return self.var_declaration();
        }

        if self.match_one(TokenType::Class).is_some() {
            return self.class_declaration();
        }

        return self.statement();
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.expect(
            "var delcaration requires an identifier",
            TokenType::Identifier,
        )?;

        let initializer = if self.match_one(TokenType::Equal).is_some() {
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

    fn class_declaration(&mut self) -> Result<Stmt, ParseError> {
        let class_name = self.expect(
            "class delcaration requires an identifier",
            TokenType::Identifier,
        )?;
        self.expect("class statement left brace", TokenType::LeftBrace)?;
        let mut methods = Vec::new();
        while let Some(t) = self.tokens.peek() {
            if t.is_err() || t.unwrap().token_type == TokenType::RightBrace {
                break;
            }
            let func = self.function(None)?;
            if func.is_anonymous() {
                return Err(ParseError::InvalidClassMethod {
                    location: func.view(),
                });
            }
            methods.push(func);
        }
        self.expect("class statement right brace", TokenType::RightBrace)?;
        Ok(Stmt::Class {
            name: class_name.try_into()?,
            methods,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_one(TokenType::Print).is_some() {
            return self.print_statement();
        }
        if self.match_one(TokenType::LeftBrace).is_some() {
            return self.block_statement();
        }
        if self.match_one(TokenType::If).is_some() {
            return self.if_statement();
        }
        if self.match_one(TokenType::While).is_some() {
            return self.while_statement();
        }
        if self.match_one(TokenType::For).is_some() {
            return self.for_statement();
        }
        if self.match_one(TokenType::Break).is_some() {
            return self.break_statement();
        }
        if self.match_one(TokenType::Continue).is_some() {
            return self.continue_statement();
        }
        if self.match_one(TokenType::Return).is_some() {
            return self.return_statement();
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.enter_loop();
        self.expect("for statement left parens", TokenType::LeftParen)?;

        let intializer = if self.match_one(TokenType::Semicolon).is_some() {
            None
        } else if self.match_one(TokenType::Var).is_some() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.match_one(TokenType::Semicolon).is_some() {
            None
        } else {
            let expr = self.expression()?;
            self.expect("for statement semicolon", TokenType::Semicolon)?;
            Some(expr)
        };

        let increment = if self.match_one(TokenType::Semicolon).is_some() {
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

        let else_block = if self.match_one(TokenType::Else).is_some() {
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

    fn break_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.tokens.last().unwrap();
        if !self.is_in_loop() {
            return Err(ParseError::InvalidLoopKeyword {
                type_str: keyword.lexeme.to_string(),
                location: keyword.pos,
            });
        }
        self.expect("unterminated break statement", TokenType::Semicolon)?;
        Ok(Stmt::Break)
    }

    fn continue_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.tokens.last().unwrap();
        if !self.is_in_loop() {
            return Err(ParseError::InvalidLoopKeyword {
                type_str: keyword.lexeme.to_string(),
                location: keyword.pos,
            });
        }
        self.expect("unterminated break statement", TokenType::Semicolon)?;
        Ok(Stmt::Break)
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.tokens.last().unwrap();
        if !self.is_in_fn() {
            return Err(ParseError::InvalidReturn {
                location: keyword.pos,
            });
        }

        if let Some(_) = self.match_one(TokenType::Semicolon) {
            return Ok(Stmt::Return { value: None });
        }

        // return only requires a terminating semi-colon for non-function expressions.
        let ret_expr = self.expression()?;
        match ret_expr {
            Expr::Function { .. } => Ok(Stmt::Return {
                value: Some(ret_expr),
            }),
            _ => {
                self.expect("unterminated return statement", TokenType::Semicolon)?;
                Ok(Stmt::Return {
                    value: Some(ret_expr),
                })
            }
        }
    }

    fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let not_terminated = |t: &'_ Token<'_>| {
            t.token_type != TokenType::RightBrace && t.token_type != TokenType::Eof
        };
        let mut statements = Vec::new();
        while let Some(_) = self.tokens.peek_next_if(not_terminated)? {
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
        match expr {
            Expr::Function { value } => Ok(desugar_function_statement(value)),
            other => {
                self.expect("unterminated expression statement", TokenType::Semicolon)?;
                Ok(Stmt::Expression { expr: other })
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logical_or()?;
        if let Some(eq) = self.match_one(TokenType::Equal) {
            let value = Box::new(self.assignment()?);
            return match expr {
                Expr::Variable { value: name } => Ok(Expr::Assignment { name, value }),
                Expr::Get { object, property } => Ok(Expr::Set {
                    object,
                    property,
                    value,
                }),
                _ => Err(ParseError::UnexpectedAssignment {
                    type_str: expr.type_str().to_string(),
                    location: eq.pos,
                }),
            };
        }

        if let Some(eq) = self.match_many(&[
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::StarEqual,
            TokenType::SlashEqual,
        ]) {
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
        while let Some(or) = self.match_one(TokenType::Or) {
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
        while let Some(and) = self.match_one(TokenType::And) {
            let rhs = self.equality()?;
            lhs = Expr::Logical {
                left: Box::new(lhs),
                op: and.try_into()?,
                right: Box::new(rhs),
            }
        }
        return Ok(lhs);
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_many(&[TokenType::BangEqual, TokenType::EqualEqual]) {
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

        while let Some(op) = self.match_many(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
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
        while let Some(op) = self.match_many(&[TokenType::Plus, TokenType::Minus]) {
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
        while let Some(op) = self.match_many(&[TokenType::Slash, TokenType::Star]) {
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
        if let Some(op) = self.match_many(&[TokenType::Bang, TokenType::Minus]) {
            Ok(Expr::Unary {
                prefix: op.try_into()?,
                value: Box::new(self.unary()?),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        while let Some(next) = self.tokens.peek() {
            match next {
                Ok(t) if t.token_type == TokenType::LeftParen => {
                    expr = self.handle_call(expr)?;
                }
                Ok(t) if t.token_type == TokenType::Dot => {
                    expr = self.handle_dot_access(expr)?;
                }
                Ok(_) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(expr)
    }

    fn handle_call(&mut self, expr: Expr) -> Result<Expr, ParseError> {
        let paren = self.tokens.next()?;
        let args = self.arguments()?;
        if args.len() > MAX_FUNC_ARGS {
            return Err(ParseError::FuncExceedMaxArgs {
                max: MAX_FUNC_ARGS,
                location: paren.pos,
            });
        }
        Ok(Expr::Call {
            callee: Callee {
                expr: Box::new(expr),
                view: paren.pos,
            },
            args,
        })
    }

    fn handle_dot_access(&mut self, expr: Expr) -> Result<Expr, ParseError> {
        let _dot = self.tokens.next()?;
        let name = self.expect("dot access missing identifier", TokenType::Identifier)?;
        Ok(Expr::Get {
            object: Box::new(expr),
            property: name.try_into()?,
        })
    }

    fn arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::with_capacity(MAX_FUNC_ARGS);
        if self.match_one(TokenType::RightParen).is_some() {
            return Ok(args);
        }
        args.push(self.expression()?);
        while self.match_one(TokenType::Comma).is_some() {
            args.push(self.expression()?);
        }
        self.expect("function call did not terminate", TokenType::RightParen)?;
        Ok(args)
    }

    fn parameters(&mut self) -> Result<Vec<Identifier>, ParseError> {
        let mut params = Vec::with_capacity(MAX_FUNC_ARGS);
        if self.match_one(TokenType::RightParen).is_some() {
            return Ok(params);
        }
        params.push(
            self.tokens
                .assert(TokenType::Identifier, "function dec")?
                .try_into()?,
        );
        while self.match_one(TokenType::Comma).is_some() {
            params.push(
                self.tokens
                    .assert(TokenType::Identifier, "function dec")?
                    .try_into()?,
            );
        }
        self.expect("function params did not terminate", TokenType::RightParen)?;
        Ok(params)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_one(TokenType::LeftParen).is_some() {
            let expr = self.expression()?;
            let _ = self.expect(
                "primary grouping did not terminate correctly",
                TokenType::RightParen,
            )?;
            return Ok(Expr::Grouping {
                expr: Box::new(expr),
            });
        }

        if let Some(fun) = self.match_one(TokenType::Fun) {
            return self.fun_expression(fun.pos);
        }

        if let Some(name) = self.match_one(TokenType::Identifier) {
            return Ok(Expr::Variable {
                value: name.try_into()?,
            });
        }

        if let Some(this) = self.match_one(TokenType::This) {
            return Ok(Expr::This {
                ident: this.try_into()?,
            });
        }

        let next_tok = self.tokens.next()?;
        let value = next_tok.try_into()?;
        Ok(Expr::Literal { value })
    }

    fn fun_expression(&mut self, marker_location: View) -> Result<Expr, ParseError> {
        Ok(Expr::Function {
            value: self.function(Some(marker_location))?,
        })
    }

    fn function(&mut self, marker_location: Option<View>) -> Result<Function, ParseError> {
        self.enter_fn();
        // if the function is anonymous then there will be no identifier after it.
        let name = if let Some(t) = self.match_one(TokenType::Identifier) {
            Some(Identifier::try_from(t)?)
        } else {
            None
        };
        // regardless of the above point, it must be followed by some params
        let begin_args = self.expect("function dec must open", TokenType::LeftParen)?;
        let params = self.parameters()?;
        // functions are required to be followed by a block scope, so we force this by doing a little look-ahead.
        let _ = self.expect("function must open to block scope", TokenType::LeftBrace)?;
        let ret = Function::new(
            name,
            params,
            Rc::new(self.block_statement()?),
            // if the caller didn't already have a place to point
            // diagnostics, then we should default to whereever the args began.
            marker_location.unwrap_or(begin_args.pos),
        );
        self.exit_fn();
        Ok(ret)
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

    fn take_done(&mut self) -> bool {
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

    fn is_in_fn(&self) -> bool {
        self.fn_cnt > 0
    }

    fn enter_loop(&mut self) {
        self.loop_cnt += 1;
    }

    fn enter_fn(&mut self) {
        self.fn_cnt += 1;
    }

    fn exit_loop(&mut self) {
        self.loop_cnt -= 1;
    }

    fn exit_fn(&mut self) {
        self.fn_cnt -= 1;
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

fn desugar_function_statement(value: Function) -> Stmt {
    if let Some(name) = value.name() {
        return Stmt::Var {
            name: name,
            initializer: Some(Expr::Function { value }),
        };
    } else {
        return Stmt::Expression {
            expr: Expr::Function { value },
        };
    }
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
