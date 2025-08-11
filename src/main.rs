use rloxv2::bytecode::instruction::OpCode;
use rloxv2::bytecode::memory::Memory;
use rloxv2::bytecode::vm::VirtualMachine;
use std::io::{self, BufWriter};
const INPUT: &str = r#"
fun fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}

var before = clock();
print fib(40);
var after = clock();
print after - before;
"#;

fn main() {
    let mut memory = Memory::new();

    memory.push_constant_f64(10.0, 0);
    for _ in 0..10_000_000 {
        memory.push_opcode(OpCode::Negate, 0);
    }
    memory.push_opcode(OpCode::Return, 0);
    let mut vm = VirtualMachine::new(Some(memory));
    match vm.interpret() {
        Ok(_) => {}
        Err(e) => println!("{e}"),
    }
}

// expression     → assignment ;

// assignment     → ( call "." )? IDENTIFIER "=" assignment
//                | logic_or ;

// logic_or       → logic_and ( "or" logic_and )* ;
// logic_and      → equality ( "and" equality )* ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;

// unary          → ( "!" | "-" ) unary | call ;
// call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
// primary        → "true" | "false" | "nil" | "this"
//                | NUMBER | STRING | IDENTIFIER | "(" expression ")"
//                | "super" "." IDENTIFIER ;
