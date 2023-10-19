extern crate core;

pub mod ast;
pub mod decl;
pub mod err;
pub mod exec;
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

use err::ExecErr;
use exec::{Key, Syms};
use log::{Level, Logger};

use crate::{
    exec::{Excecuter, Scope, Sym},
    sym::Req,
};

lazy_static::lazy_static! {
    pub static ref LOGGER: Logger = Logger { level: Level::None };
}

pub fn repl() {
    let mut inp = String::new();
    // let mut analyzer = sem::Analyzer::new();
    println!("Tipis Repl");

    let mut syms = Syms::new(Vec::new());

    loop {
        print!(">>\t| ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        inp = inp.trim().to_string();
        println!("{inp}");

        match run(&syms, &inp) {
            Res::None => {}
            _ => {
                inp.clear();
                continue;
            }
        }

        let mut syn = syntax::Syntax::new(&inp.as_bytes());
        let res = syn.parse();
        println!("{:#?}", res);

        if let Ok(tok) = res {
            let name = tok.name();
            let res = syms.add(Sym {
                name: tok.name().unwrap(),
                ty: tok.ty(),
                kind: tok.kind(),
                scope: Scope::Global,
                reqs: Req::None,
                val: tok,
            });
            println!("{}", syms.has(&Key(name.unwrap(), Scope::Global)));
            if let Err(err) = res {
                println!("{:#?}", err);
            }
        }

        inp.clear();
    }
}

enum Res {
    DidAction,
    None,
    InvalidArgs,
    NotFound,
    Err(ExecErr),
}

fn run(syms: &Syms, cmd: &str) -> Res {
    let parts = cmd.split(' ').collect::<Vec<&str>>();
    if parts.len() == 0 {
        return Res::None;
    }

    match parts[0].to_lowercase().as_str() {
        "clear" | "c" | "cls" => {
            print!("{}[2J", 27 as char);
            Res::DidAction
        }
        "quit" | "q" | "exit" | "e" => std::process::exit(0),
        "show" | "s" => {
            for sym in syms.symbols.values() {
                println!("{:#?}", sym);
            }
            Res::DidAction
        }
        "make" | "mk" | "m" => {
            println!("{:#?}", parts);
            if parts.len() < 2 {
                return Res::InvalidArgs;
            }
            let sym = syms.get(&Key(parts[1].to_string(), Scope::Global));
            println!("{:#?}", sym);
            if let None = sym {
                return Res::NotFound;
            }
            let sym = sym.unwrap();
            let res = match sym {
                Sym {
                    val: ast::Ast::Dir(dir),
                    ..
                } => Excecuter::dir(&syms, dir.clone()),
                Sym {
                    val: ast::Ast::File(file),
                    ..
                } => Excecuter::file(syms, std::path::PathBuf::from("examples"), file.clone()),
                _ => return Res::InvalidArgs,
            };
            println!("{:#?}", res);
            match res {
                Ok(_) => Res::DidAction,
                Err(err) => Res::Err(err.into()),
            }
        }
        _ => Res::None,
    }
}
