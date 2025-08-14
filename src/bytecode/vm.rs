use crate::bytecode::compiler::Compiler;
use crate::bytecode::instruction::OpCode;
use crate::bytecode::memory::Memory;
use std::ops::Neg;

pub struct VmOptions {
    pub memory: Memory,
    pub source: String,
}

impl VmOptions {
    pub fn new(source: String) -> Self {
        Self {
            source,
            memory: Memory::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmState {
    Done,
    Error,
    Running,
    Pending,
}

pub struct VirtualMachine {
    memory: Memory,
    pc: usize,
    state: VmState,
    source: String,
}

impl VirtualMachine {
    pub fn new(options: VmOptions) -> Self {
        Self {
            memory: options.memory,
            source: options.source,
            pc: 0,
            state: VmState::Pending,
        }
    }

    pub fn interpret(&mut self) -> Result<(), String> {
        let compiler = Compiler::new(&self.source, &mut self.memory);
        if let Err(e) = compiler.compile() {
            println!("{e}");
            return Err(e.to_string());
        }

        self.start();
        while self.running() {
            let op = self.fetch_opcode();
            match op {
                OpCode::Return => {
                    self.stop();
                }
                OpCode::Constant => self.handle_constant()?,
                OpCode::ConstantLong => self.handle_constant_long()?,
                OpCode::Negate => self.handle_negate()?,
                OpCode::Add => self.handle_add()?,
                OpCode::Sub => self.handle_sub()?,
                OpCode::Mul => self.handle_mul()?,
                OpCode::Div => self.handle_div()?,
                OpCode::Unknown => {
                    self.error();
                }
            };
        }
        Ok(())
    }

    fn handle_constant(&mut self) -> Result<(), String> {
        let idx = self.fetch_u8();
        let val = self.memory.constant_get(idx as usize);
        self.memory.stack_push(val);
        Ok(())
    }

    fn handle_constant_long(&mut self) -> Result<(), String> {
        let idx = self.fetch_u16();
        let val = self.memory.constant_get(idx as usize);
        self.memory.stack_push(val);
        Ok(())
    }

    fn handle_negate(&mut self) -> Result<(), String> {
        let val = self.memory.stack_pop().neg();
        self.memory.stack_push(val);
        Ok(())
    }

    fn handle_add(&mut self) -> Result<(), String> {
        let b = self.memory.stack_pop();
        let a = self.memory.stack_pop();
        self.memory.stack_push(a + b);
        Ok(())
    }

    fn handle_sub(&mut self) -> Result<(), String> {
        let b = self.memory.stack_pop();
        let a = self.memory.stack_pop();
        self.memory.stack_push(a - b);
        Ok(())
    }

    fn handle_mul(&mut self) -> Result<(), String> {
        let b = self.memory.stack_pop();
        let a = self.memory.stack_pop();
        self.memory.stack_push(a * b);
        Ok(())
    }

    fn handle_div(&mut self) -> Result<(), String> {
        let b = self.memory.stack_pop();
        let a = self.memory.stack_pop();
        self.memory.stack_push(a / b);
        Ok(())
    }

    pub fn start(&mut self) {
        self.state = VmState::Running;
    }

    pub fn stop(&mut self) {
        self.state = VmState::Done;
    }

    pub fn running(&mut self) -> bool {
        self.state == VmState::Running
    }

    pub fn error(&mut self) -> bool {
        self.state == VmState::Error
    }

    #[inline]
    pub fn fetch_u8(&mut self) -> u8 {
        let op = self.memory.text_get(self.pc);
        self.pc += 1;
        op
    }

    #[inline]
    pub fn fetch_u16(&mut self) -> u16 {
        let op = self.memory.text_get_u16(self.pc);
        self.pc += 2;
        op
    }

    /// Retrieves the instruction at the specified location in the code.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[cfg(debug_assertions)]
    pub fn fetch_opcode(&mut self) -> OpCode {
        let (op, debug_info) = self.memory.text_get_debug(self.pc);
        println!("stack: {:?}", &self.memory.stack_slice()[..]);
        println!("{}", debug_info);
        self.pc += 1;
        op
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn fetch_opcode(&mut self) -> OpCode {
        OpCode::from(self.fetch_u8())
    }
}
