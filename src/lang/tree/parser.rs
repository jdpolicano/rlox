use super::ast::Expr;
use super::error::ParseError;
use crate::lang::tokenizer::scanner::Scanner;
use crate::lang::tokenizer::token::{Token, TokenType};
use std::iter::{Iterator, Peekable};

type ParseResult = Result<Expr, ParseError>;

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
}

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(src),
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        self.expression()
    }

    fn expression(&mut self) -> ParseResult {
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

    fn comparison(&mut self) -> ParseResult {
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

    fn term(&mut self) -> ParseResult {
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

    fn factor(&mut self) -> ParseResult {
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

    fn unary(&mut self) -> ParseResult {
        if let Some(op) = self.match_unary() {
            Ok(Expr::Unary {
                prefix: op.try_into()?,
                value: Box::new(self.unary()?),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParseResult {
        if let Some(_) = self.match_one(TokenType::LeftParen) {
            let expr = self.expression()?;
            let _ = self.expect(
                "primary grouping did not terminate correctly",
                TokenType::RightParen,
            )?;
            return Ok(Expr::Grouping {
                expr: Box::new(expr),
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

    /// recover from a panic state by reading through until we hit the end of the stream, or alternatively a semi-colon terminator.
    fn recover(&mut self) {
        while let Some(result) = self.tokens.peek() {
            match result {
                Ok(toke) if toke.token_type == TokenType::Semicolon => break,
                Ok(toke) if toke.token_type == TokenType::Eof => break,
                _ => {
                    let _ = self.tokens.next();
                }
            }
        }
    }

    // fn expect_one_of(
    //     &mut self,
    //     msg: &'static str,
    //     ts: &[TokenType],
    // ) -> Result<Token<'a>, ParseError> {
    //     let toke = self.tokens.next()?;
    //     if ts.contains(&toke.token_type) {
    //         Ok(toke)
    //     } else {
    //         Err(ParseError::UnexpectedToken {
    //             expected: ts[0], // Placeholder since multiple types are expected
    //             recieved: toke.token_type,
    //             msg,
    //         })
    //     }
    // }
}
