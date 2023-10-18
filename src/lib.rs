extern crate core;

pub mod ast;
pub mod decl;
pub mod err;
pub mod expr;
pub mod sem;
pub mod sym;
pub mod syntax;
pub mod token;
pub mod visitor;
#[macro_use]
pub mod log;
pub mod charset;
pub mod lexer;

use std::io::Write;

use log::{Level, Logger};

lazy_static::lazy_static! {
    pub static ref LOGGER: Logger = Logger { level: Level::None };
}

pub fn repl() {
    let mut inp = String::new();
    println!("Tipis Repl");
    // let mut analyzer = sem::Analyzer::new();

    loop {
        print!(">>\t| ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        let mut lex = lexer::Lexer::new(&inp.as_bytes());
        let mut syn = syntax::Syntax::new(&inp.as_bytes());
        let res = syn.take();
        match res {
            Ok(tok) => {
                println!("{:#?}", tok);
            }
            Err(err) => println!("{:#?}", err),
        }
        // let decl = parser.parse_decl();
        // match decl {
        //     Ok(decl) => {
        //         let _ = analyzer.analyze(decl);
        //         println!("{:#?}", analyzer);
        //     }
        //     Err(e) => println!("{:#?}", e),
        // }
        inp.clear();
    }
}
