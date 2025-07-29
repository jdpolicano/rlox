use rloxv2::interpreter::lox::Lox;
use rloxv2::lang::tree::parser::Parser;
const INPUT: &str = r#"
var a = 1.5;
var b = 2.3;
print a + b + c
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
