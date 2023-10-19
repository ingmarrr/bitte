use crate::{
    ast::{Ast, AstKind, Dir, File, Let, Ty},
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

    pub fn parse(&mut self) -> Result<Ast, Trace<'a, SynErr>> {
        let tok = self.take()?;
        match tok.kind {
            TokKind::Keyword(Keyword::Dir) => Ok(Ast::Dir(self.parse_dir()?)),
            TokKind::Keyword(Keyword::Let) => Ok(Ast::Let(self.parse_let()?)),
            TokKind::Keyword(Keyword::File) => Ok(Ast::File(self.parse_file()?)),
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
            let mut next = self.look_ahead()?;

            if let TokKind::Symbol(Symbol::At) = next.kind {
                self.take()?;
                let dirname = self.assert(TokKind::Ident)?;
                let ext = match self.look_ahead()?.kind {
                    TokKind::Symbol(Symbol::Dot) => {
                        self.take()?;
                        if let Ok(Token {
                            kind: TokKind::Ident,
                            ..
                        }) = self.look_ahead()
                        {
                            let res = Some(self.take()?);
                            next = self.look_ahead()?;
                            res
                        } else {
                            None
                        }
                    }
                    TokKind::Closer(Closer::RCurly) => {
                        files.push(Ast::Ref(AstKind::File, dirname.val.unwrap().to_owned()));
                        return Ok(Dir {
                            name: std::path::PathBuf::from(name.val.unwrap()),
                            children,
                            files,
                        });
                    }
                    _ => {
                        return Err(Trace::new(
                            self.lx.look_ahead()?.src,
                            SynErr::Expected(
                                "`:`".into(),
                                self.lx.look_ahead()?.kind.to_string(),
                                self.lx.look_ahead()?.src.to_string(),
                            ),
                        ))
                    }
                };
                let refdir = dirname.val.unwrap().to_owned()
                    + &ext
                        .map(|t| ".".to_owned() + t.val.unwrap())
                        .unwrap_or("".to_owned());
                files.push(Ast::Ref(AstKind::File, refdir))
            }

            match next.kind {
                TokKind::Closer(Closer::RCurly) | TokKind::EOF => {
                    return Ok(Dir {
                        name: std::path::PathBuf::from(name.val.unwrap()),
                        children,
                        files,
                    })
                }
                TokKind::Ident => {}
                _ => {
                    return Err(Trace::new(
                        next.src,
                        SynErr::Expected(
                            "struct- or file name".into(),
                            next.kind.to_string(),
                            next.src.to_string(),
                        ),
                    ))
                }
            }

            let closer = self.look_ahead()?;
            match closer.kind {
                TokKind::Symbol(Symbol::Comma) => {
                    children.push(Ast::Dir(Dir {
                        name: std::path::PathBuf::from(next.val.unwrap()),
                        children: Vec::new(),
                        files: Vec::new(),
                    }));
                    self.take()?;
                    self.take()?;
                }
                TokKind::Closer(Closer::RCurly) => {
                    children.push(Ast::Dir(Dir {
                        name: std::path::PathBuf::from(next.val.unwrap()),
                        children: Vec::new(),
                        files: Vec::new(),
                    }));
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
                    children.push(Ast::Dir(self.parse_dir()?));
                }
                TokKind::Symbol(Symbol::Colon) | TokKind::Symbol(Symbol::Dot) => {
                    files.push(Ast::File(self.parse_file()?));
                    if let Ok(Token {
                        kind: TokKind::Symbol(Symbol::Comma),
                        ..
                    }) = self.look_ahead()
                    {
                        self.take()?;
                    }
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
        let ext = match self.look_ahead()?.kind {
            TokKind::Symbol(Symbol::Dot) => {
                self.take()?;
                if let Ok(Token {
                    kind: TokKind::Ident,
                    ..
                }) = self.look_ahead()
                {
                    Some(self.take()?)
                } else {
                    None
                }
            }
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
        let name = name.val.unwrap().to_owned()
            + &ext
                .map(|t| ".".to_owned() + t.val.unwrap())
                .unwrap_or("".to_owned());

        let fi = match self.take()? {
            Token {
                kind: TokKind::Literal(Literal::String(StringTy::Literal)),
                val,
                ..
            } => File {
                name,
                content: Box::new(Ast::Lit(val.unwrap().to_owned())),
            },
            Token {
                kind: TokKind::Ident,
                val,
                ..
            } => File {
                name,
                content: Box::new(Ast::Ref(AstKind::Let, val.unwrap().to_owned())),
            },
            _ => {
                return Err(Trace::new(
                    self.lx.look_ahead()?.src,
                    SynErr::Expected(
                        "string or identifier".into(),
                        self.lx.look_ahead()?.kind.to_string(),
                        self.lx.look_ahead()?.src.to_string(),
                    ),
                ))
            }
        };

        if let Ok(Token {
            kind: TokKind::Symbol(Symbol::Semi),
            ..
        }) = self.look_ahead()
        {
            self.take()?;
        }

        Ok(fi)
    }

    pub fn parse_let(&mut self) -> Result<Let, Trace<'a, SynErr>> {
        let name = self.assert(TokKind::Ident)?;
        let _ = self.assert(TokKind::Symbol(Symbol::Equal))?;
        let expr = self.assert(TokKind::Literal(Literal::String(StringTy::Literal)))?;

        Ok(Let {
            name: name.val.unwrap().to_owned(),
            ty: Ty::Str,
            expr: expr.val.unwrap().to_owned(),
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
