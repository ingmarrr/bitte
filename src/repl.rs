use std::io::Write;

fn main() {
    let mut inp = String::new();
    println!("Tp Repl");

    loop {
        print!(">>\t");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        let lex = tipis::lex::Lexer::new(&inp);
        let mut parser = tipis::parse::Parser::new(lex);
        let decls = parser.parse();
        if let Err(e) = decls {
            println!("{:#?}", e);
        } else {
            let mut analyzer = tipis::sem::Analyzer::new();
            let decls = analyzer.analyze(decls.unwrap());
            println!("{:#?}", analyzer);
        }
        inp.clear();
    }
}
