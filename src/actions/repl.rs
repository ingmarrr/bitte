use std::io::Write;

use crate::{
    ast::{self, Ast, AstKind, Req, Ty},
    consts,
    err::ExecErr,
    exec::{Exec, Key, Scope, Sym, Syms},
    syntax,
};

pub fn repl() {
    let mut inp = String::new();
    println!("Tipis Repl");

    let mut syms = Syms::new(Vec::new());

    loop {
        // let arrows = "\x1b[36m>>>\x1b[0m";
        let arrows = ">>>";
        print!("{}{}{} ", consts::BLUE, arrows, consts::RESET);
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();

        inp = inp.trim().to_string();
        println!("{inp}");

        match run(&mut syms, &inp) {
            Res::None | Res::Commit => {}
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
                    main: tok.main(),
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
            println!("{}", res.unwrap_err().dump_err(inp.as_bytes()));
        }

        inp.clear();
    }
}

enum Res {
    Commit,
    Continue,
    DidAction,
    InvalidArgs,
    NotFound,
    Err(ExecErr),
    None,
}

fn run(syms: &mut Syms, cmd: &str) -> Res {
    let parts = cmd.split(' ').collect::<Vec<&str>>();
    if parts.len() == 0 {
        return Res::None;
    }

    match parts[0].to_lowercase().as_str() {
        "commit" => Res::Commit,
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
                        main: false,
                        name: arg_parts[0].to_string(),
                        ty: Ty::String,
                        kind: AstKind::Lit,
                        scope: Scope::Global,
                        reqs: Vec::new(),
                        val: Ast::Req(Req {
                            name: arg_parts[0].to_string(),
                            ty: Ty::String,
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
                } => Exec::dir(&syms, dir.clone()),
                Sym {
                    val: ast::Ast::File(file),
                    ..
                } => Exec::file(
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
        _ => Res::Continue,
    }
}
