use std::error::Error;

use crate::{
    ast::Prog,
    decl::{Decl, Required},
    err::Trace,
    token::Token,
};

pub struct Parser<'a, E>
where
    E: Error + 'static,
{
    toks: Vec<Token<'a>>,
    errs: Vec<Trace<'a, E>>,
    pos: usize,
}

impl<'a, E> Parser<'a, E>
where
    E: Error + 'static,
{
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            toks: tokens,
            errs: Vec::new(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Prog, Trace<'a, E>> {
        let requirements = self.parse_requireds();
        if let Err(e) = requirements {
            return Err(e);
        }

        Ok(Prog {
            requirements: vec![],
        })
    }

    fn take(&mut self) -> Option<Token<'a>> {
        if self.pos > self.toks.len() {
            return None;
        }

        let res = Some(self.toks[self.pos]);
        self.pos += 1;
        return res;
    }

    /// Returns the next token.
    fn peek(&mut self, n: usize) -> Option<Token<'a>> {
        if self.pos + n > self.toks.len() {
            return None;
        }

        Some(self.toks[self.pos + n])
    }

    fn parse_requireds(&mut self) -> Result<Vec<Required>, Trace<'a, E>> {}
}
