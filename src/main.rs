use rloxv2::interpreter::lox::Lox;
use rloxv2::lang::tree::parser::Parser;
use rloxv2::lang::tree::resolver::Resolver;
const INPUT: &str = r#"
var a = "string";
print a.nothing;
"#;

fn main() {
    let mut parser = Parser::new(&INPUT);
    parser.parse();
    if parser.had_errors() {
        let errors = parser.take_errors();
        println!("{}", errors[0]);
        errors[0].print_code_block(&INPUT);
        return;
    }
    let mut res = Resolver::new();
    let mut lox = Lox::new();
    let stmts = parser.take_statements();
    for stmt in &stmts {
        if let Err(e) = stmt.accept(&mut res) {
            println!("{}", e);
            break;
        }
    }
    if let Err(e) = lox.interpret(stmts) {
        println!("{}", e);
    };
}

// use rloxv2::bytecode::instruction::OpCode;
// use rloxv2::bytecode::memory::Memory;
// use rloxv2::bytecode::vm::VirtualMachine;
// use rloxv2::interpreter::lox;
// use std::io::{self, BufWriter};
// const INPUT: &str = r#"
// fun fib(n) {
//   if (n < 2) return n;
//   return fib(n - 1) + fib(n - 2);
// }

// var before = clock();
// print fib(40);
// var after = clock();
// print after - before;
// "#;

// fn main() {
//     let interpreter = Lox::new();
//     let mut memory = Memory::new();

//     memory.push_constant_f64(10.0, 0);
//     for _ in 0..10_000_000 {
//         memory.push_opcode(OpCode::Negate, 0);
//     }
//     memory.push_opcode(OpCode::Return, 0);
//     let mut vm = VirtualMachine::new(Some(memory));
//     match vm.interpret() {
//         Ok(_) => {}
//         Err(e) => println!("{e}"),
//     }
// }

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
