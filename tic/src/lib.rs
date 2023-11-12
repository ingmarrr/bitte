extern crate core;

pub mod actions;
pub mod ast;
pub mod charset;
pub mod consts;
pub mod err;
pub mod exec;
pub mod fifo;
pub mod lexer;
pub mod stack;
pub mod syntax;
pub mod token;

use ast::Expr;
use ast::Lit;

pub use actions::check::check;
pub use actions::check::return_check;
pub use actions::init::init;
pub use actions::make::make;
pub use actions::publish::publish;
pub use actions::repl::repl;

pub struct Template {
    pub name: String,
    pub body: String,
}

pub fn args(args: Vec<String>) -> Option<Vec<(String, Expr)>> {
    let mut res = Vec::new();
    for arg in args {
        let mut arg = arg.split('=');
        let name = arg.next().unwrap().to_string();
        let val = Expr::Lit(Lit::String(arg.next().unwrap().to_string()));
        res.push((name, val));
    }
    Some(res)
}
