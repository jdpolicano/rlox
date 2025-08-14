use crate::bytecode::instruction::OpCode;
use crate::bytecode::memory::Memory;
use crate::bytecode::object::LoxObject;
use crate::lang::tokenizer::span::Span;
use crate::lang::tree::ast::{
    BinaryOperator, Callee, Expr, Function, Identifier, Literal, LogicalOperator, PropertyName,
    Stmt, UnaryPrefix,
};
use crate::lang::visitor::Visitor;
use thiserror::Error;

pub type CodeGenResult = Result<(), CodeGenError>;

#[derive(Debug, Clone, Error)]
pub enum CodeGenError {
    #[error("feature '{feature}' not yet supported")]
    UnsupportedFeature { feature: String },
}

pub struct CodeGen<'a> {
    memory: &'a mut Memory,
    current_stmt_span: Span,
}

impl<'a> CodeGen<'a> {
    pub fn new(memory: &'a mut Memory) -> Self {
        Self {
            memory,
            current_stmt_span: Span::new(0, 0),
        }
    }

    pub fn code_gen(mut self, stmts: &[Stmt]) -> Result<(), CodeGenError> {
        for stmt in stmts {
            // this is so all of the bytecode associated with a given statment are grouped together.
            // and share the same "span". At runtime it is not as important that we pick out the exact
            // token that caused the error, but instead to just point out the line where the error occured.
            self.current_stmt_span = stmt.span();
            stmt.accept(&mut self)?
        }
        self.memory
            .text_push_opcode(OpCode::Return, Span::new(0, 0));
        Ok(())
    }

    fn push_constant(&mut self, obj: LoxObject) {
        let constant_idx = self.memory.constant_len();
        self.memory.constant_push(obj);
        if constant_idx < u8::MAX as usize {
            self.memory
                .text_push_opcode(OpCode::Constant, self.current_stmt_span);
            self.memory
                .text_push_u8(constant_idx as u8, self.current_stmt_span);
        } else {
            debug_assert!(
                constant_idx < u16::MAX as usize,
                "number of constants in memory is way too much."
            );
            let idx_u16 = constant_idx as u16;
            self.memory
                .text_push_opcode(OpCode::ConstantLong, self.current_stmt_span);
            self.memory
                .text_push_slice(&idx_u16.to_be_bytes(), self.current_stmt_span);
        }
    }
}

impl<'a> Visitor<CodeGenResult, Expr, Stmt> for CodeGen<'a> {
    fn visit_binary(&mut self, left: &Expr, op: BinaryOperator, right: &Expr) -> CodeGenResult {
        left.accept(self)?;
        right.accept(self)?;
        self.memory
            .text_push_opcode(bin_op_to_opcode(op)?, self.current_stmt_span);
        Ok(())
    }

    fn visit_logical(&mut self, left: &Expr, op: LogicalOperator, right: &Expr) -> CodeGenResult {
        Ok(())
    }

    fn visit_grouping(&mut self, expr: &Expr) -> CodeGenResult {
        expr.accept(self)
    }

    fn visit_literal(&mut self, value: &Literal) -> CodeGenResult {
        let obj = LoxObject::from(value);
        self.push_constant(obj);
        Ok(())
    }

    fn visit_unary(&mut self, prefix: UnaryPrefix, expr: &Expr) -> CodeGenResult {
        let obj = expr.accept(self)?;
        Ok(())
    }

    fn visit_variable(&mut self, ident: &Identifier) -> CodeGenResult {
        Ok(())
    }

    fn visit_assignment(&mut self, ident: &Identifier, value: &Expr) -> CodeGenResult {
        Ok(())
    }

    fn visit_call(&mut self, callee: &Callee, args: &[Expr]) -> CodeGenResult {
        Ok(())
    }

    fn visit_function(&mut self, value: &Function) -> CodeGenResult {
        Ok(())
    }

    fn visit_get(&mut self, object: &Expr, property: &PropertyName) -> CodeGenResult {
        Ok(())
    }

    fn visit_set(&mut self, object: &Expr, property: &PropertyName, value: &Expr) -> CodeGenResult {
        Ok(())
    }

    fn visit_this(&mut self, ident: &Identifier) -> CodeGenResult {
        Ok(())
    }

    fn visit_break_statement(&mut self) -> CodeGenResult {
        Ok(())
    }

    fn visit_continue_statment(&mut self) -> CodeGenResult {
        Ok(())
    }

    fn visit_return_statment(&mut self, value: Option<&Expr>) -> CodeGenResult {
        Ok(())
    }

    fn visit_expression_statement(&mut self, expr: &Expr) -> CodeGenResult {
        expr.accept(self)
    }

    fn visit_print_statement(&mut self, expr: &Expr) -> CodeGenResult {
        Ok(())
    }

    fn visit_var_statement(
        &mut self,
        ident: &Identifier,
        initializer: Option<&Expr>,
    ) -> CodeGenResult {
        Ok(())
    }

    fn visit_block_statement(&mut self, statements: &[Stmt]) -> CodeGenResult {
        Ok(())
    }

    fn visit_if_statement(
        &mut self,
        condition: &Expr,
        if_block: &Stmt,
        else_block: Option<&Stmt>,
    ) -> CodeGenResult {
        Ok(())
    }

    fn visit_while_statement(&mut self, condition: &Expr, block: &Stmt) -> CodeGenResult {
        Ok(())
    }

    fn visit_class_statement(
        &mut self,
        name: &Identifier,
        super_class: Option<&Expr>,
        methods: &[Function],
    ) -> CodeGenResult {
        Ok(())
    }
}

fn bin_op_to_opcode(b: BinaryOperator) -> Result<OpCode, CodeGenError> {
    match b {
        BinaryOperator::Slash(_) => Ok(OpCode::Div),
        BinaryOperator::Minus(_) => Ok(OpCode::Sub),
        BinaryOperator::Plus(_) => Ok(OpCode::Add),
        BinaryOperator::Star(_) => Ok(OpCode::Mul),
        other => Err(CodeGenError::UnsupportedFeature {
            feature: other.to_string(),
        }),
    }
}
