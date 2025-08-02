use rloxv2::interpreter::lox::Lox;
use rloxv2::lang::tree::parser::Parser;
const INPUT: &str = r#"
var a1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
var a2 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2";
var a3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3";
var a4 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4";
var a5 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa5";
var a6 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa6";
var a7 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa7";
var a8 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa8";

var n = 1500;

fun constants() {
  var i = 0;
  while (i < n) {
      i = i + 1;
      a1; a1; a1; a2; a1; a3; a1; a4; a1; a5; a1; a6; a1; a7; a1; a8;
      a2; a1; a2; a2; a2; a3; a2; a4; a2; a5; a2; a6; a2; a7; a2; a8;
      a3; a1; a3; a2; a3; a3; a3; a4; a3; a5; a3; a6; a3; a7; a3; a8;
      a4; a1; a4; a2; a4; a3; a4; a4; a4; a5; a4; a6; a4; a7; a4; a8;
      a5; a1; a5; a2; a5; a3; a5; a4; a5; a5; a5; a6; a5; a7; a5; a8;
      a6; a1; a6; a2; a6; a3; a6; a4; a6; a5; a6; a6; a6; a7; a6; a8;
      a7; a1; a7; a2; a7; a3; a7; a4; a7; a5; a7; a6; a7; a7; a7; a8;
      a8; a1; a8; a2; a8; a3; a8; a4; a8; a5; a8; a6; a8; a7; a8; a8;
    }
}

fun equality() {
  var i = 0;
  while (i < n) {
    i = i + 1;
    a1 == a1; a1 == a2; a1 == a3; a1 == a4; a1 == a5; a1 == a6; a1 == a7; a1 == a8;
    a2 == a1; a2 == a2; a2 == a3; a2 == a4; a2 == a5; a2 == a6; a2 == a7; a2 == a8;
    a3 == a1; a3 == a2; a3 == a3; a3 == a4; a3 == a5; a3 == a6; a3 == a7; a3 == a8;
    a4 == a1; a4 == a2; a4 == a3; a4 == a4; a4 == a5; a4 == a6; a4 == a7; a4 == a8;
    a5 == a1; a5 == a2; a5 == a3; a5 == a4; a5 == a5; a5 == a6; a5 == a7; a5 == a8;
    a6 == a1; a6 == a2; a6 == a3; a6 == a4; a6 == a5; a6 == a6; a6 == a7; a6 == a8;
    a7 == a1; a7 == a2; a7 == a3; a7 == a4; a7 == a5; a7 == a6; a7 == a7; a7 == a8;
    a8 == a1; a8 == a2; a8 == a3; a8 == a4; a8 == a5; a8 == a6; a8 == a7; a8 == a8;
  }
}

var loopStart = clock();
constants();
var loopTime = clock() - loopStart;

var start = clock();
equality();
var elapsed = clock() - start;

print "loop";
print loopTime;
print "elapsed";
print elapsed;
print "equals";
print elapsed - loopTime;
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
