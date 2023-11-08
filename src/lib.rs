extern crate core;

pub mod args;
pub mod ast;
pub mod check;
pub mod consts;
pub mod err;
pub mod exec;
pub mod fifo;
pub mod init;
pub mod local;
pub mod make;
pub mod publish;
pub mod repl;
pub mod stack;
pub mod syntax;
pub mod token;
#[macro_use]
pub mod log;
pub mod charset;
pub mod lexer;

use log::{Level, Logger};

lazy_static::lazy_static! {
    pub static ref LOGGER: Logger = Logger { level: Level::None };
}

pub use args::args;
pub use check::check;
pub use check::return_check;
pub use init::init;
pub use make::make;
pub use publish::publish;
pub use repl::repl;

pub struct Template {
    pub name: String,
    pub body: String,
}

//file hey(name: str) {"Hey there, Mr {$name$}"}; dir foo { hello { "world": "# Hello World" }, foo { #hey(name: "bob") } }
