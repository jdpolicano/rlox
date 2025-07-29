use rloxv2::interpreter::lox::Lox;
use rloxv2::lang::tree::parser::Parser;
const INPUT: &str = r#"
true and false
"#;

fn main() {
    let mut parser = Parser::new(&INPUT);
    match parser.parse() {
        Ok(expr) => {
            let lox = Lox::new();
            match lox.visit(expr) {
                Ok(v) => println!("{}", v),
                Err(e) => println!("{}", e),
            }
        }
        Err(e) => println!("{e}"),
    };
}
