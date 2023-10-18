use crate::{
    ast::{Dir, File},
    err::{SynErr, Trace},
    lexer::Lexer,
    token::{Closer, Keyword, Literal, Opener, StringTy, Symbol, TokKind, Token},
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

    pub fn take(&mut self) -> Result<Token<'a>, Trace<'a, SynErr>> {
        Ok(self.lx.next_token()?)
    }

    pub fn look_ahead(&mut self) -> Result<Token<'a>, Trace<'a, SynErr>> {
        Ok(self.lx.look_ahead()?)
    }

    pub fn parse(&mut self) -> Result<Dir, Trace<'a, SynErr>> {
        let tok = self.take()?;
        match tok.kind {
            TokKind::Keyword(Keyword::Struct) => self.parse_dir(),
            TokKind::Keyword(Keyword::Let) => self.parse_file().map(|f| Dir {
                name: ".".into(),
                children: Vec::new(),
                files: vec![f],
            }),
            _ => Err(Trace::new(
                self.lx.look_ahead()?.src,
                SynErr::Expected(
                    "struct or file".into(),
                    self.lx.look_ahead()?.kind.to_string(),
                    self.lx.look_ahead()?.src.to_string(),
                ),
            )),
        }
    }

    pub fn parse_dir(&mut self) -> Result<Dir, Trace<'a, SynErr>> {
        let name = self.assert(TokKind::Ident)?;

        let next = self.look_ahead()?;
        match next.kind {
            TokKind::Opener(Opener::LCurly) => {
                let _ = self.take()?;
            }
            TokKind::Closer(Closer::RCurly)
            | TokKind::Symbol(Symbol::Semi)
            | TokKind::Symbol(Symbol::Comma) => {
                return Ok(Dir {
                    name: std::path::PathBuf::from(name.val.unwrap()),
                    children: Vec::new(),
                    files: Vec::new(),
                })
            }
            _ => {
                return Err(Trace::new(
                    next.src,
                    SynErr::Expected(
                        "`{` or `}`".into(),
                        next.kind.to_string(),
                        next.src.to_string(),
                    ),
                ));
            }
        }

        let mut children = Vec::new();
        let mut files = Vec::new();

        loop {
            let next = self.look_ahead()?;

            if let TokKind::Closer(Closer::RCurly) = next.kind {
                return Ok(Dir {
                    name: std::path::PathBuf::from(name.val.unwrap()),
                    children,
                    files,
                });
            }

            if next.kind != TokKind::Ident {
                return Err(Trace::new(
                    next.src,
                    SynErr::Expected(
                        "struct- or file name".into(),
                        next.kind.to_string(),
                        next.src.to_string(),
                    ),
                ));
            }

            let closer = self.look_ahead()?;
            match closer.kind {
                TokKind::Symbol(Symbol::Comma) => {
                    children.push(Dir {
                        name: std::path::PathBuf::from(next.val.unwrap()),
                        children: Vec::new(),
                        files: Vec::new(),
                    });
                    self.take()?;
                    self.take()?;
                }
                TokKind::Closer(Closer::RCurly) => {
                    children.push(Dir {
                        name: std::path::PathBuf::from(next.val.unwrap()),
                        children: Vec::new(),
                        files: Vec::new(),
                    });
                    self.take()?;
                    self.take()?;
                    // Important to return here, otherwise next iteration we might encounter a
                    // `\0` and return an error when in fact everything was totally fine.
                    return Ok(Dir {
                        name: std::path::PathBuf::from(name.val.unwrap()),
                        children,
                        files,
                    });
                }
                TokKind::Opener(Opener::LCurly) => {
                    children.push(self.parse_dir()?);
                }
                TokKind::Symbol(Symbol::Colon) | TokKind::Symbol(Symbol::Dot) => {
                    files.push(self.parse_file()?)
                }
                _ => {
                    return Err(Trace::new(
                        next.src,
                        SynErr::Expected(
                            "`:` or `{`".into(),
                            closer.kind.to_string(),
                            closer.src.to_string(),
                        ),
                    ))
                }
            }
        }
    }

    pub fn parse_file(&mut self) -> Result<File, Trace<'a, SynErr>> {
        let name = self.assert(TokKind::Ident)?;
        let ext = match self.take()?.kind {
            TokKind::Symbol(Symbol::Dot) => Some(self.take()?),
            TokKind::Symbol(Symbol::Colon) => None,
            _ => {
                return Err(Trace::new(
                    self.lx.look_ahead()?.src,
                    SynErr::Expected(
                        "file extension or colon".into(),
                        self.lx.look_ahead()?.kind.to_string(),
                        self.lx.look_ahead()?.src.to_string(),
                    ),
                ))
            }
        };

        let _ = self.assert(TokKind::Symbol(Symbol::Colon))?;

        let content = self
            .assert(TokKind::Literal(Literal::String(StringTy::Literal)))?
            .val
            .unwrap()
            .to_owned();

        Ok(File {
            name: name.val.unwrap().to_owned()
                + &ext
                    .map(|t| ".".to_owned() + t.val.unwrap())
                    .unwrap_or("".to_owned()),
            content,
        })
    }

    fn assert(&mut self, kind: TokKind) -> Result<Token<'a>, Trace<'a, SynErr>> {
        let tok = self.lx.next_token()?;
        if tok.kind != kind {
            return Err(Trace::new(
                tok.src,
                SynErr::Expected(kind.to_string(), tok.kind.to_string(), tok.src.to_string()),
            ));
        }
        Ok(tok)
    }
}
