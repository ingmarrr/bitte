use crate::{
    exec::{self, Syms},
    syntax::Syntax,
};

use super::local::Local;

pub fn make(name: String, args: Vec<(String, String)>) {
    let repo = match Local::new() {
        Ok(local) => local,
        Err(err) => {
            println!("Sqlite Error: {}", err);
            return;
        }
    };

    let template = match repo.get(&name) {
        Ok(template) => template,
        Err(err) => {
            println!("Sqlite Error: {}", err);
            return;
        }
    };

    let mut syms = Syms::new(args);
    let mut syn = Syntax::new(template.body.as_bytes());
    let res = match syn.parse_all() {
        Ok(res) => res,
        Err(err) => {
            println!("{}", err.dump_err(template.body.as_bytes()));
            return;
        }
    };

    if let Err(err) = syms.add_all_ast(res) {
        println!("{:#?}", err);
        return;
    };

    let main = match syms.main() {
        Some(main) => main,
        None => {
            println!("No main function found");
            return;
        }
    };

    match exec::Exec::run(&syms, main) {
        Ok(_) => println!("Done"),
        Err(err) => {
            println!("{:#?}", err);
        }
    }
}
