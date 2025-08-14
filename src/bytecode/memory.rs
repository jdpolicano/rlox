use crate::bytecode::instruction::{OpCode, OpConversionError};
use crate::bytecode::object::LoxObject;
use crate::lang::tokenizer::span::Span;
use std::io;

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
#[derive(Debug)]
pub struct Memory {
    // the src code as u8s (can be cast to opcodes)
    text: Vec<u8>,
    // the line info for the test segement, run-length encoded.
    spans: Vec<(Span, usize)>,
    // the operating stack
    stack: Vec<LoxObject>,
    // the program constants
    constants: Vec<LoxObject>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            spans: Vec::new(),
            stack: Vec::new(),
            constants: Vec::new(),
        }
    }

    #[inline]
    pub fn text_len(&self) -> usize {
        self.text.len()
    }

    #[inline]
    pub fn text_is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[inline]
    pub fn text_get(&self, loc: usize) -> u8 {
        debug_assert!(
            loc < self.text.len(),
            "index would go out of bounds on code segment"
        );
        self.text[loc]
    }

    #[inline]
    pub fn text_get_u16(&self, loc: usize) -> u16 {
        debug_assert!(
            loc + 1 < self.text.len(),
            "index would go out of bounds on code segment"
        );
        let b1 = self.text[loc];
        let b2 = self.text[loc + 1];
        u16::from_le_bytes([b1, b2])
    }

    pub fn text_get_debug(&self, code_idx: usize) -> (OpCode, String) {
        let prefix = self.format_line_prefix(code_idx);
        self.decode_opcode(code_idx, &prefix)
    }

    #[inline]
    pub fn text_push_u8(&mut self, v: u8, span: Span) {
        self.text.push(v);
        if let Some(last) = self.spans.last_mut() {
            if last.0 == span {
                last.1 += 1;
                return;
            }
        }
        self.spans.push((span, 1));
    }

    /// Pushes an opcode into the code memory, associating it with a specific line number.
    #[inline]
    pub fn text_push_opcode(&mut self, op: OpCode, span: Span) {
        self.text_push_u8(op as u8, span);
    }

    #[inline]
    pub fn text_push_slice(&mut self, v: &[u8], span: Span) {
        self.text.extend_from_slice(v);
        if let Some(last) = self.spans.last_mut() {
            if last.0 == span {
                last.1 += v.len();
                return;
            }
        }
        self.spans.push((span, v.len()));
    }

    fn text_get_span(&self, text_idx: usize) -> (Span, bool) {
        let mut count = 0;
        for (span, n) in &self.spans {
            if count <= text_idx && text_idx < count + n {
                return if count == text_idx {
                    (*span, true)
                } else {
                    (*span, false)
                };
            }
            count += n;
        }
        panic!("Instruction index out of bounds: {}", text_idx);
    }

    /// Pushes a value onto the stack
    #[inline]
    pub fn stack_push(&mut self, val: LoxObject) {
        self.stack.push(val)
    }

    /// Pops a value off the stack.
    ///
    /// # Returns
    /// Returns `Some(LoxObject)` if the stack is not empty, otherwise `None`.
    #[inline]
    pub fn stack_pop(&mut self) -> LoxObject {
        debug_assert!(self.stack.len() > 0, "pop from stack invalid with len 0");
        self.stack.pop().unwrap()
    }

    /// returns a range of values from the stack.
    ///
    /// # Returns
    /// Returns `Some(LoxObject)` if the stack is not empty, otherwise `None`.
    #[inline]
    pub fn stack_slice(&self) -> &[LoxObject] {
        &self.stack[..]
    }

    /// returns a range of values from the stack.
    ///
    /// # Returns
    /// Returns `Some(LoxObject)` if the stack is not empty, otherwise `None`.
    #[inline]
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    /// Retrieves the constant value at the specified location.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[inline]
    pub fn constant_get(&self, loc: usize) -> LoxObject {
        debug_assert!(
            loc < self.constants.len(),
            "index would go out of bounds on constants"
        );
        self.constants.get(loc).unwrap().clone()
    }

    /// Retrieves the constant value at the specified location.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[inline]
    pub fn constant_push(&mut self, v: LoxObject) {
        self.constants.push(v)
    }

    /// Retrieves the constant value at the specified location.
    ///
    /// # Panics
    /// Panics if the location is out of bounds.
    #[inline]
    pub fn constant_len(&mut self) -> usize {
        self.constants.len()
    }

    /// Formats the line prefix for debugging.
    fn format_line_prefix(&self, code_idx: usize) -> String {
        let (span, is_first) = self.text_get_span(code_idx);
        let line_num_str = format!("{}", span);

        if is_first {
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
        let op = OpCode::from(self.text_get(code_idx));
        match op {
            OpCode::Constant => {
                let cidx = self.text_get(code_idx + 1);
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
                let part1 = self.text_get(code_idx + 1);
                let part2 = self.text_get(code_idx + 2);
                let slice = [part1, part2];
                let cidx = u16::from_le_bytes(slice) as usize;
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

    /// Dumps the assembly representation of the bytecode stored in memory to the console.
    pub fn dump_assm(&mut self) -> Result<(), MemoryError> {
        if self.text_is_empty() {
            return Ok(());
        }
        println!("=======begin dump=======");
        let mut idx = 0;
        while idx < self.text.len() {
            let (op, debug) = self.text_get_debug(idx);
            println!("{}", debug);
            idx += op.num_args() + 1;
        }
        Ok(())
    }
}
