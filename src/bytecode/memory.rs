use crate::bytecode::instruction::{OpCode, OpConversionError};
use crate::bytecode::object::LoxObject;
use std::io;

struct LineInfo {
    line_num: usize,
    is_first: bool,
}

/// Represents errors that can occur during memory operations.
#[derive(Debug)]
pub enum MemoryError {
    InvalidOpCode,
    EmptyStream,
    OutOfBounds,
    Io(io::Error),
}

impl From<OpConversionError> for MemoryError {
    fn from(_: OpConversionError) -> Self {
        Self::InvalidOpCode
    }
}

impl From<io::Error> for MemoryError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Represents the memory structure used in the virtual machine, including code, line encodings,
/// RAM, and constants.
pub struct Memory {
    // the src code as u8s (can be cast to opcodes)
    code_src: Vec<u8>,
    // the line info, run-length encoded.
    code_lines: Vec<(usize, usize)>,
    stack: Vec<LoxObject>,
    constants: Vec<LoxObject>,
}

impl Memory {
    #[inline]
    pub fn code_len(&self) -> usize {
        self.code_src.len()
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            code_src: Vec::new(),
            code_lines: Vec::new(),
            stack: Vec::new(),
            constants: Vec::new(),
        }
    }

    #[inline]
    fn code_is_empty(&self) -> bool {
        self.code_src.is_empty()
    }

    #[inline]
    fn code_get(&self, loc: usize) -> u8 {
        debug_assert!(
            loc < self.code_src.len(),
            "index would go out of bounds on code segment"
        );
        self.code_src[loc]
    }

    #[inline]
    fn code_push_u8(&mut self, v: u8, line: usize) {
        self.code_src.push(v);
        if let Some((l, cnt)) = self.code_lines.last_mut() {
            if line == *l {
                *cnt += 1;
                return;
            }
        }
        self.code_lines.push((line, 1));
    }

    #[inline]
    fn code_push_slice(&mut self, v: &[u8], line: usize) {
        self.code_src.extend_from_slice(v);
        if let Some((l, cnt)) = self.code_lines.last_mut() {
            if line == *l {
                *cnt += v.len();
                return;
            }
        }
        self.code_lines.push((line, v.len()));
    }

    fn code_get_line(&self, instruction_idx: usize) -> LineInfo {
        let mut count = 0;
        for (line, n) in &self.code_lines {
            if count <= instruction_idx && instruction_idx < count + n {
                return if count == instruction_idx {
                    LineInfo {
                        line_num: *line,
                        is_first: true,
                    }
                } else {
                    LineInfo {
                        line_num: *line,
                        is_first: false,
                    }
                };
            }
            count += n;
        }
        panic!("Instruction index out of bounds: {}", instruction_idx);
    }

    /// Pushes an opcode into the code memory, associating it with a specific line number.
    #[inline]
    pub fn push_opcode(&mut self, op: OpCode, line: usize) {
        self.code_push_u8(op as u8, line);
    }

    /// Pushes a constant value into the constants memory and generates the appropriate opcode
    /// to reference it in the code memory.
    pub fn push_constant_f64(&mut self, constant: f64, line: usize) {
        let constant_index = self.constants.len();
        self.constants.push(LoxObject::Number(constant));
        if constant_index < u8::MAX as usize {
            self.code_push_u8(OpCode::Constant as u8, line);
            self.code_push_u8(constant_index as u8, line);
        } else {
            self.code_push_u8(OpCode::ConstantLong as u8, line);
            self.code_push_slice(&(constant_index as u16).to_be_bytes(), line);
        }
    }

    /// Pushes a value onto the stack
    #[inline]
    pub fn push_stack(&mut self, val: LoxObject) {
        self.stack.push(val)
    }

    /// Pops a value off the stack.
    ///
    /// # Returns
    /// Returns `Some(LoxObject)` if the stack is not empty, otherwise `None`.
    pub fn pop_stack(&mut self) -> LoxObject {
        debug_assert!(self.stack.len() > 0, "pop from stack invalid with len 0");
        self.stack.pop().unwrap()
    }

    /// Retrieves the instruction at the specified location in the code.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[cfg(debug_assertions)]
    pub fn fetch_opcode(&mut self, loc: usize) -> OpCode {
        let (op, debug_info) = self.fetch_code_debug(loc);
        println!("{}", debug_info);
        println!("stack: {:?}", &self.stack[..]);
        op
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn fetch_opcode(&self, loc: usize) -> OpCode {
        OpCode::from(self.code_get(loc))
    }

    /// Retrieves the constant value at the specified location.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[inline]
    pub fn fetch_constant(&self, loc: usize) -> LoxObject {
        debug_assert!(
            loc < self.constants.len(),
            "index would go out of bounds on constants"
        );
        self.constants.get(loc).unwrap().clone()
    }

    /// Efficiently fetches a u8 value from the code memory.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[inline]
    pub fn fetch_u8(&self, loc: usize) -> u8 {
        self.code_get(loc)
    }

    /// Efficiently fetches a u8 value from the code memory and casts it to usize.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[inline]
    pub fn fetch_u8_as_usize(&self, loc: usize) -> usize {
        self.code_get(loc) as usize
    }

    /// Efficiently fetches a u16 value from the code memory at the specified location.
    ///
    /// # Panics
    /// Panics if the location is out of bounds or if there are not enough bytes to read a u16.
    #[inline]
    pub fn fetch_u16(&self, loc: usize) -> u16 {
        debug_assert!(
            loc + 1 < self.code_src.len(),
            "index would go out of bounds on code segment for u16"
        );
        u16::from_be_bytes([self.code_src[loc], self.code_src[loc + 1]])
    }

    /// Efficiently fetches a u16 value from the code memory at the specified location and converts to usize
    ///
    /// # Panics
    /// Panics if the location is out of bounds or if there are not enough bytes to read a u16.
    #[inline]
    pub fn fetch_u16_usize(&self, loc: usize) -> usize {
        debug_assert!(
            loc + 1 < self.code_src.len(),
            "index would go out of bounds on code segment for u16"
        );
        u16::from_be_bytes([self.code_src[loc], self.code_src[loc + 1]]) as usize
    }

    /// Formats the line prefix for debugging.
    fn format_line_prefix(&self, code_idx: usize) -> String {
        let line_info = self.code_get_line(code_idx);
        let line_num_str = format!("{}", line_info.line_num);

        if line_info.is_first {
            format!("{:08} @{}", code_idx, line_num_str)
        } else {
            let padding = std::iter::repeat(" ")
                .take(line_num_str.len())
                .collect::<String>();
            format!("{:08} {}|", code_idx, padding)
        }
    }

    /// Decodes the opcode and formats it for debugging.
    fn decode_opcode(&self, code_idx: usize, prefix: &str) -> (OpCode, String) {
        let op = OpCode::from(self.code_get(code_idx));
        match op {
            OpCode::Constant => {
                let cidx = self.code_get(code_idx + 1);
                (
                    op,
                    format!(
                        "{} {:?} -> {}",
                        prefix,
                        OpCode::Constant,
                        self.constants[cidx as usize]
                    ),
                )
            }
            OpCode::ConstantLong => {
                let part1 = self.code_get(code_idx + 1);
                let part2 = self.code_get(code_idx + 2);
                let slice = [part1, part2];
                let cidx = u16::from_be_bytes(slice) as usize;
                (
                    op,
                    format!(
                        "{} {:?} -> {}",
                        prefix,
                        OpCode::ConstantLong,
                        self.constants[cidx]
                    ),
                )
            }
            OpCode::Return => (op, format!("{} {:?}", prefix, OpCode::Return)),
            _ => (op, format!("{} {:?}", prefix, op)),
        }
    }

    /// This function takes in a line number and an opcode and formats
    /// it specifically as the first occurrence of this line number.
    /// This is to make it easier to read the dump and see which ops originated
    /// from the same line.
    fn fetch_code_debug(&self, code_idx: usize) -> (OpCode, String) {
        let prefix = self.format_line_prefix(code_idx);
        self.decode_opcode(code_idx, &prefix)
    }

    /// Dumps the assembly representation of the bytecode stored in memory to the console.
    pub fn dump_assm(&mut self) -> Result<(), MemoryError> {
        if self.code_is_empty() {
            return Ok(());
        }
        println!("=======begin dump=======");
        let mut idx = 0;
        while idx < self.code_src.len() {
            let (op, debug) = self.fetch_code_debug(idx);
            println!("{}", debug);
            idx += op.num_args() + 1;
        }
        Ok(())
    }
}
