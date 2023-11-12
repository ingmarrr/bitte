use std::io::Write;

use crate::{
    consts,
    err::ExecErr,
    exec::{Exec, Key, Scope, Sym, Syms},
    syntax,
};

pub fn repl() {
    let mut inp = String::new();
    let mut buf = String::new();
    tilog::init_logger(tilog::Config::default().with_level(tilog::Level::Debug));
    tilog::info!("Tipis Repl");

    let mut syms = Syms::new(Vec::new());

    loop {
        let arrows = ">>>";
        print!("{}{}{} ", consts::BLUE, arrows, consts::RESET);
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();

        inp = inp.trim().to_string();

        match run(&mut syms, &inp) {
            Res::None | Res::Commit => {
                inp.clear();
            }
            Res::Continue => {
                buf.push_str(&inp);
                inp.clear();
                continue;
            }
            _ => {
                buf.clear();
                inp.clear();
                continue;
            }
        }

        let src = buf.clone();
        let mut syn = syntax::Syntax::new(&src.as_bytes());
        let res = syn.parse_all();
        println!("{:#?}", res);

        if let Ok(toks) = res {
            for tok in toks.into_iter() {
                let res = syms.add(Sym {
                    scope: Scope::Global,
                    val: tok.clone(),
                });
                if let Err(err) = res {
                    println!("{:#?}", err);
                }
            }
        } else {
            println!("{}", res.unwrap_err().dump_err(buf.as_bytes()));
        }

        buf.clear();
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
    if cmd == "" {
        return Res::Commit;
    }
    let parts = cmd.split(' ').collect::<Vec<&str>>();
    if parts.len() == 0 {
        return Res::None;
    }

    match parts[0].to_lowercase().as_str() {
        "quit" | "q" | "exit" | "e" => std::process::exit(0),
        "show" | "s" => {
            for sym in syms.symbols.values() {
                println!("{:#?}", sym);
            }
            Res::DidAction
        }
        "make" | "m" => {
            if parts.len() < 2 {
                return Res::InvalidArgs;
            }
            let sym = syms.get(&Key(parts[1].to_string(), Scope::Global));
            println!("{:#?}", sym);
            if let Some(sy) = sym {
                let res = Exec::run(syms, sy.val.clone(), Vec::new());
                println!("{:#?}", res);
                return match res {
                    Ok(_) => Res::DidAction,
                    Err(err) => Res::Err(err.into()),
                };
            }
            Res::NotFound
        }
        _ => Res::Continue,
    }
}
