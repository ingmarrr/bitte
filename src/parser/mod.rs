use crate::{lexer::Lexer, ast::{Def, Kw}, token::CharToken};

use self::error::ParseResult;


pub mod error;
#[cfg(test)]
mod test;

pub struct Parser<'a> {
    lx: Lexer<'a>
}


impl<'a> Parser<'a> {

    pub fn new(lx: Lexer<'a>) -> Parser<'a> {
        Parser { lx }
    }

    fn parse_pattern(&mut self, pattern: Pattern) -> ParseResult<Def> {
    
    }
}

pub type Pattern = Vec<TokenOpt>;

pub enum TokenOpt {
    Kw(Kw),
    Char(CharToken),
    Ident(String),
    String(String),
}
