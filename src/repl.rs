use std::io::Write;

// use bitte::lexer::Lexer;
// use bitte::parser::Parser;

fn main() {
    let mut inp = String::new();
    println!("Tp Repl");

    loop {
        print!(">> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        // let mut lex = Lexer::new(&inp);
        // let mut parser = Parser::new(&mut lex);
        // let def = parser.parse();
        // println!("Program: {:#?}", def);
        inp.clear();
    }
}
