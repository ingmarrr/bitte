use std::io::Write;

fn main() {
    let mut inp = String::new();
    println!("Tp Repl");

    loop {
        print!(">>\t");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        let mut lex = bitte::lex::Lex::new(&inp);
        let tok = lex.next_token();
        println!("<<\t{:?}", tok);
        // let mut parser = Parser::new(&mut lex);
        // let def = parser.parse();
        // println!("Program: {:#?}", def);
        inp.clear();
    }
}
