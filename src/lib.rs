extern crate core;

pub mod ast;
pub mod decl;
pub mod err;
pub mod expr;
pub mod lex;
pub mod parse;
pub mod sem;
pub mod sym;
pub mod token;
#[macro_use]
pub mod log;
pub mod tok;
pub mod charset;
pub mod lexer;

use std::io::Write;

use log::{Level, Logger};

lazy_static::lazy_static! {
    pub static ref LOGGER: Logger = Logger { level: Level::None };
}

pub fn repl() {
    let mut inp = String::new();
    println!("Tp Repl");
    let mut analyzer = sem::Analyzer::new();

    loop {
        print!(">>\t| ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        let lex = lex::Lexer::new(&inp);
        let mut parser = parse::Parser::new(lex);
        let decl = parser.parse_decl();
        match decl {
            Ok(decl) => {
                let _ = analyzer.analyze(decl);
                println!("{:#?}", analyzer);
            }
            Err(e) => println!("{:#?}", e),
        }
        inp.clear();
    }
}
