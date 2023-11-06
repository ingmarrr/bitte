extern crate core;

pub mod ast;
pub mod decl;
pub mod err;
pub mod exec;
pub mod expr;
pub mod fifo;
pub mod sem;
pub mod stack;
pub mod sym;
pub mod syntax;
pub mod token;
#[macro_use]
pub mod log;
pub mod charset;
pub mod lexer;

use std::io::Write;

use err::ExecErr;
use exec::{Key, Syms};
use log::{Level, Logger};

use crate::{
    ast::{Ast, AstKind, Req, Ty},
    exec::{Excecutor, Scope, Sym},
};

lazy_static::lazy_static! {
    pub static ref LOGGER: Logger = Logger { level: Level::None };
}

pub fn repl() {
    let mut inp = String::new();
    println!("Tipis Repl");

    let mut syms = Syms::new(Vec::new());

    loop {
        print!(">>\t| ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        if inp == "\n" {
            inp.clear();
            continue;
        }
        inp = inp.trim().to_string();
        println!("{inp}");

        match run(&mut syms, &inp) {
            Res::None => {}
            _ => {
                inp.clear();
                continue;
            }
        }

        let mut syn = syntax::Syntax::new(&inp.as_bytes());
        let res = syn.parse_all();
        println!("{:#?}", res);

        if let Ok(toks) = res {
            for tok in toks.into_iter() {
                let res = syms.add(Sym {
                    name: tok.alias().or(tok.name()).unwrap(),
                    ty: tok.ty(),
                    kind: tok.kind(),
                    scope: Scope::Global,
                    reqs: Vec::new(),
                    val: tok.clone(),
                });
                if let Err(err) = res {
                    println!("{:#?}", err);
                }
            }
        } else {
            println!("{}", res.unwrap_err().display_line(inp.as_bytes()));
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

fn run(syms: &mut Syms, cmd: &str) -> Res {
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
            } else if parts.len() > 2 {
                for part in &parts[2..] {
                    let arg_parts = part.split('=').collect::<Vec<&str>>();
                    if arg_parts.len() != 2 {
                        return Res::InvalidArgs;
                    }
                    let _ = syms.add(Sym {
                        name: arg_parts[0].to_string(),
                        ty: Ty::Str,
                        kind: AstKind::Lit,
                        scope: Scope::Global,
                        reqs: Vec::new(),
                        val: Ast::Req(Req {
                            name: arg_parts[0].to_string(),
                            ty: Ty::Str,
                            expr: arg_parts[1].to_string(),
                        }),
                    });
                }
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
                } => Excecutor::dir(&syms, dir.clone()),
                Sym {
                    val: ast::Ast::File(file),
                    ..
                } => Excecutor::file(
                    syms,
                    std::path::PathBuf::from("examples"),
                    file.clone(),
                    Vec::new(),
                ),
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
