use std::error::Error;

use crate::{
    decl::Decl,
    err::{SynErr, Trace},
    lexer::Lexer,
    sym::Key,
    token::TokKind,
};

pub struct Syntax<'a> {
    lx: Lexer<'a>,
    errs: Vec<Trace<'a, SynErr>>,
}

impl<'a> Syntax<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            lx: Lexer::new(src),
            errs: Vec::new(),
        }
    }

    pub fn take(&mut self) -> Result<TokKind, Trace<'a, SynErr>> {
        let tok = self.lx.next_token()?;
        Ok(tok.kind)
    }

    pub fn parse(&mut self) -> Result<Vec<Decl>, Trace<'a, SynErr>> {
        let mut decls = Vec::new();

        use TokKind::*;
        loop {
            let tok = self.lx.look_ahead()?;
            if let EOF = tok.kind {
                break;
            }
        }

        Ok(decls)
    }
}
