use crate::bytecode::instruction::OpCode;
use crate::bytecode::memory::Memory;
use crate::bytecode::object::LoxObject;
use std::ops::Neg;

macro_rules! binary_op {
    ($vm:expr, $op:tt) => {
        {
            let b = $vm.pop_stack();
            let a = $vm.pop_stack();
            $vm.push_stack((a $op b));
        }
    };
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
}

impl VirtualMachine {
    pub fn new(memory: Option<Memory>) -> Self {
        Self {
            memory: memory.unwrap_or(Memory::new()),
            pc: 0,
            state: VmState::Pending,
        }
    }

    pub fn interpret(&mut self) -> Result<(), String> {
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
        let val = self.read_constant();
        println!("{}", val);
        self.memory.push_stack(val);
        Ok(())
    }

    fn handle_constant_long(&mut self) -> Result<(), String> {
        let val = self.read_constant_long();
        println!("{}", val);
        self.memory.push_stack(val);
        Ok(())
    }

    fn handle_negate(&mut self) -> Result<(), String> {
        let val = self.memory.pop_stack().neg();
        self.memory.push_stack(val);
        Ok(())
    }

    fn handle_add(&mut self) -> Result<(), String> {
        binary_op!(self.memory, +);
        Ok(())
    }

    fn handle_sub(&mut self) -> Result<(), String> {
        binary_op!(self.memory, -);
        Ok(())
    }

    fn handle_mul(&mut self) -> Result<(), String> {
        binary_op!(self.memory, *);
        Ok(())
    }

    fn handle_div(&mut self) -> Result<(), String> {
        binary_op!(self.memory, /);
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

    pub fn fetch_opcode(&mut self) -> OpCode {
        let op = self.memory.fetch_opcode(self.pc);
        self.pc += 1;
        op
    }

    pub fn fetch_u8(&mut self) -> u8 {
        let op = self.memory.fetch_u8(self.pc);
        self.pc += 1;
        op
    }

    pub fn fetch_u16(&mut self) -> u16 {
        let op = self.memory.fetch_u16(self.pc);
        self.pc += 2;
        op
    }

    pub fn fetch_u16_usize(&mut self) -> usize {
        let op = self.memory.fetch_u16_usize(self.pc);
        self.pc += 2;
        op
    }

    pub fn fetch_u8_usize(&mut self) -> usize {
        let o = self.memory.fetch_u8_as_usize(self.pc);
        self.pc += 1;
        o
    }

    pub fn read_constant(&mut self) -> LoxObject {
        let idx = self.fetch_u8_usize();
        self.memory.fetch_constant(idx)
    }

    pub fn read_constant_long(&mut self) -> LoxObject {
        let idx = self.fetch_u16() as usize;
        self.memory.fetch_constant(idx)
    }
}
