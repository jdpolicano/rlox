use rloxv2::interpreter::lox::Lox;
use rloxv2::lang::tree::parser::Parser;
use rloxv2::lang::tree::resolver::Resolver;
const INPUT: &str = r#"

"#;

fn main() {
    let mut parser = Parser::new(&INPUT);
    parser.parse();
    if parser.had_errors() {
        return;
    }
    let mut res = Resolver::new();
    let mut lox = Lox::new();
    let stmts = parser.take_statements();
    for stmt in &stmts {
        if let Err(e) = stmt.accept(&mut res) {
            println!("{e}");
            break;
        }
    }
    if let Err(e) = lox.interpret(stmts) {
        println!("{}", e);
    };
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
