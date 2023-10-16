use std::error::Error;

use crate::{decl::Decl, err::Trace, lexer::Lexer};

pub struct Syntax<'a, E>
where
    E: Error + 'static,
{
    lx: Lexer<'a>,
    errs: Vec<Trace<'a, E>>,
}

impl<'a, E> Syntax<'a, E>
where
    E: Error + 'static,
{
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            lx: Lexer::new(src),
            errs: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Decl>, Trace<'a, E>> {
        Ok(vec![])
    }
}
