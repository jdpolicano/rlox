use crate::bytecode::codegen::{CodeGen, CodeGenError};
use crate::bytecode::memory::Memory;
use crate::lang::tree::ast::Stmt;
use crate::lang::tree::error::ParseError;
use crate::lang::tree::parser::Parser;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum CompileError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    CodeGenError(#[from] CodeGenError),
}

pub struct Compiler<'a> {
    memory: &'a mut Memory,
    src: &'a str,
}

impl<'a> Compiler<'a> {
    pub fn new(src: &'a str, memory: &'a mut Memory) -> Self {
        Self { memory, src }
    }

    pub fn compile(self) -> Result<(), CompileError> {
        let stmts = self.parse()?;
        let memory = self.setup_memory_image(&stmts)?;
        Ok(memory)
    }

    fn parse(&self) -> Result<Vec<Stmt>, CompileError> {
        let mut parser = Parser::new(self.src);
        parser.parse();
        if parser.had_errors() {
            let errors = parser.take_errors();
            for e in &errors {
                println!("{e}");
            }
            Err(errors[0].clone().into())
        } else {
            Ok(parser.take_statements())
        }
    }

    /// prepares the memory for exectution by:
    /// 1. compiling the actual tree into the correct bytecode
    /// 2. intializing any constants / globals that are needed ahead of time.
    fn setup_memory_image(mut self, stmts: &[Stmt]) -> Result<(), CompileError> {
        let mut codegen = CodeGen::new(&mut self.memory);
        for stmt in stmts {
            stmt.accept(&mut codegen)?;
        }
        Ok(())
    }
}
