use rloxv2::interpreter::lox::Lox;
use rloxv2::lang::tree::parser::Parser;
const INPUT: &str = r#"
var i = 0;
while(i < 1000000) {
    print i;
}
"#;

fn main() {
    let mut parser = Parser::new(&INPUT);
    parser.parse();
    if parser.had_errors() {
        //println!("{:#?}", parser.take_statements());
        return;
    }
    let mut lox = Lox::new();
    match lox.interpret(parser.take_statements()) {
        Err(e) => {
            println!("{}", e)
        }
        _ => {}
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
