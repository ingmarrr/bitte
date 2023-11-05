use crate::{
    ast::{Ast, AstKind, Dir, Expr, File, Let, Lit, Ref, Ty},
    err::{SynErr, Trace},
    lexer::Lexer,
    stack::Stack,
    token::{Closer, Keyword, Literal, Opener, Symbol, TokKind, Token},
};

pub struct Syntax<'a> {
    lx: Lexer<'a>,
    _errs: Vec<Trace<'a, SynErr>>,
}

impl<'a> Syntax<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            lx: Lexer::new(src),
            _errs: Vec::new(),
        }
    }

    pub fn take(&mut self) -> Result<Token<'a>, Trace<'a, SynErr>> {
        Ok(self.lx.next_token()?)
    }

    pub fn look_ahead(&mut self) -> Result<Token<'a>, Trace<'a, SynErr>> {
        Ok(self.lx.look_ahead()?)
    }

    pub fn look_ahead_one(&mut self) -> Result<Token<'a>, Trace<'a, SynErr>> {
        Ok(self.lx.look_ahead_one()?)
    }

    pub fn parse_all(&mut self) -> Result<Vec<Ast>, Trace<'a, SynErr>> {
        let mut asts = Vec::new();
        while let Ok(tok) = self.look_ahead_one() {
            if tok.kind == TokKind::EOF {
                break;
            }
            asts.push(self.parse()?);
        }
        Ok(asts)
    }

    pub fn parse(&mut self) -> Result<Ast, Trace<'a, SynErr>> {
        let tok = self.take()?;
        match tok.kind {
            TokKind::Keyword(Keyword::Dir) => Ok(Ast::Dir(self.parse_dir_lit()?)),
            TokKind::Keyword(Keyword::Let) => Ok(Ast::Let(self.parse_let()?)),
            TokKind::Keyword(Keyword::File) => Ok(Ast::File(self.parse_file_lit()?)),
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

    pub fn parse_dir_lit(&mut self) -> Result<Dir, Trace<'a, SynErr>> {
        let (ident, mut path) =
            match self.assert_union(&[TokKind::Opener(Opener::DQuote), TokKind::Ident])? {
                Token {
                    kind: TokKind::Opener(Opener::DQuote),
                    ..
                } => {
                    let v = self
                        .assert(TokKind::Literal(Literal::String))?
                        .val
                        .unwrap()
                        .to_owned();
                    let _ = self.assert(TokKind::Closer(Closer::DQuote));
                    (v.clone(), Some(v))
                }
                Token {
                    kind: TokKind::Ident,
                    val,
                    ..
                } => {
                    let mut buf = val.unwrap().to_owned();
                    while let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Slash)) {
                        buf.push('/');
                        buf.push_str(self.assert(TokKind::Ident)?.val.unwrap());
                    }
                    (buf, None)
                }
                _ => unreachable!(),
            };

        let params = self.parse_params()?;
        if path.is_none() {
            path = if let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Colon)) {
                if let TokKind::Opener(Opener::DQuote) = self.look_ahead_one()?.kind {
                    Some(self.parse_raw_string()?)
                } else {
                    let mut buf = self.assert(TokKind::Ident)?.val.unwrap().to_owned();
                    while let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Slash)) {
                        buf.push('/');
                        buf.push_str(self.assert(TokKind::Ident)?.val.unwrap());
                    }
                    Some(buf)
                }
            } else {
                Some(ident.clone())
            }
        };

        let di = Dir {
            name: std::path::PathBuf::from(path.unwrap()),
            alias: ident,
            params,
            children: self.parse_children()?,
        };

        Ok(di)
    }

    pub fn parse_children(&mut self) -> Result<Vec<Ast>, Trace<'a, SynErr>> {
        if let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Semi)) {
            return Ok(Vec::new());
        }

        let _ = self.assert(TokKind::Opener(Opener::LCurly))?;
        if let Ok(_) = self.consume_if(TokKind::Closer(Closer::RCurly)) {
            return Ok(Vec::new());
        }

        let mut children = Vec::new();
        while let Ok(tok) = self.consume_if_union(&[
            TokKind::Symbol(Symbol::Pound),
            TokKind::Symbol(Symbol::At),
            TokKind::Symbol(Symbol::Bang),
            TokKind::Ident,
            TokKind::Opener(Opener::DQuote),
        ]) {
            match tok.kind {
                TokKind::Symbol(Symbol::Pound)
                | TokKind::Symbol(Symbol::Bang)
                | TokKind::Symbol(Symbol::At) => {
                    children.push(Ast::Ref(Ref {
                        name: self.assert(TokKind::Ident)?.val.unwrap().to_owned(),
                        kind: tok.kind.symbol_to_ast_kind().unwrap(),
                        args: self.parse_args()?,
                    }));
                    if let Ok(_) = self.consume_if(TokKind::Closer(Closer::RCurly)) {
                        break;
                    }
                }
                TokKind::Opener(Opener::DQuote) => {
                    let name = self.assert(TokKind::Literal(Literal::String))?.val.unwrap();
                    let _ = self.assert(TokKind::Closer(Closer::DQuote))?;
                    if let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Colon)) {
                        children.push(Ast::File(File {
                            params: Vec::new(),
                            name: name.to_owned(),
                            alias: name.to_owned(),
                            content: self.parse_string()?,
                        }))
                    } else {
                        children.push(Ast::File(File {
                            params: Vec::new(),
                            name: name.to_owned(),
                            alias: name.to_owned(),
                            content: Vec::new(),
                        }))
                    }
                    if let Ok(_) = self.consume_if(TokKind::Closer(Closer::RCurly)) {
                        break;
                    }
                }
                TokKind::Ident => {
                    let next = self.assert_union(&[
                        TokKind::Symbol(Symbol::Comma),
                        TokKind::Symbol(Symbol::Colon),
                        TokKind::Closer(Closer::RCurly),
                    ])?;

                    match next.kind {
                        TokKind::Symbol(Symbol::Comma) => {
                            children.push(Ast::Dir(Dir {
                                params: Vec::new(),
                                name: tok.val.unwrap().to_owned().into(),
                                alias: tok.val.unwrap().to_owned(),
                                children: self.parse_children()?,
                            }));
                        }
                        TokKind::Symbol(Symbol::Colon) => {
                            if let Token {
                                kind: TokKind::Ident,
                                ..
                            } = self.look_ahead_one()?
                            {
                                children.push(Ast::File(File {
                                    params: Vec::new(),
                                    name: tok.val.unwrap().to_owned(),
                                    alias: tok.val.unwrap().to_owned(),
                                    content: vec![Expr::Ref(Ref {
                                        name: self.assert(TokKind::Ident)?.val.unwrap().to_owned(),
                                        kind: AstKind::Let,
                                        args: self.parse_args()?,
                                    })],
                                }))
                            }
                            children.push(Ast::File(File {
                                params: Vec::new(),
                                name: tok.val.unwrap().to_owned(),
                                alias: tok.val.unwrap().to_owned(),
                                content: self.parse_string()?,
                            }))
                        }
                        TokKind::Closer(Closer::RCurly) => {
                            break;
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(children)
    }

    // #[rustfmt::skip]
    // pub fn parse_dir_lit(&mut self) -> Result<Dir, Trace<'a, SynErr>> {
    //     let name = self.assert(TokKind::Ident)?;

    //     let mut tok = self.assert_union(&[
    //         TokKind::Opener(Opener::LParen),
    //         TokKind::Opener(Opener::LCurly),
    //         TokKind::Symbol(Symbol::Semi),
    //         TokKind::Symbol(Symbol::Colon),
    //     ])?;

    //     let params = self.parse_params()?;
    //     if params.len() > 0 {
    //         tok = self.assert_union(&[
    //             TokKind::Opener(Opener::LCurly),
    //             TokKind::Symbol(Symbol::Semi),
    //         ])?;
    //     }

    //     let alias = if let TokKind::Symbol(Symbol::Colon) = tok.kind {
    //         let alias = self.assert(TokKind::Ident)?;
    //         tok = self.assert_union(&[
    //             TokKind::Opener(Opener::LCurly),
    //             TokKind::Symbol(Symbol::Semi),
    //         ])?;
    //         alias.val.unwrap().to_owned()
    //     } else {
    //         name.val.unwrap().to_owned()
    //     };

    //     return Ok(Dir {
    //         params,
    //         name: std::path::PathBuf::from(name.val.unwrap()),
    //         children: if tok.kind == TokKind::Symbol(Symbol::Semi) { Vec::new() } else { self.parse_children()? },
    //         alias,
    //     });
    // }

    // #[rustfmt::skip]
    // pub fn parse_dir(&mut self) -> Result<Dir, Trace<'a, SynErr>> {
    //     let name = self.assert(TokKind::Ident)?;

    //     let tok = self.assert_union(&[
    //         TokKind::Opener(Opener::LCurly),
    //         TokKind::Closer(Closer::RCurly),
    //         TokKind::Symbol(Symbol::Comma),
    //     ])?;

    //     if let TokKind::Opener(Opener::LCurly) = tok.kind {
    //         return Ok(Dir {
    //             params: Vec::new(),
    //             name: std::path::PathBuf::from(name.val.unwrap()),
    //             children: self.parse_children()?,
    //             alias: name.val.unwrap().to_owned()
    //         });
    //     }

    //     return Ok(Dir {
    //         params: Vec::new(),
    //         name: std::path::PathBuf::from(name.val.unwrap()),
    //         children: Vec::new(),
    //         alias: name.val.unwrap().to_owned()
    //     });
    // }

    // #[rustfmt::skip]
    // pub fn parse_children(&mut self) -> Result<Vec<Ast>, Trace<'a, SynErr>> {
    //     let mut children = Vec::new();
    //     loop {
    //         let next = self.look_ahead()?;
    //         match next.kind {
    //             TokKind::EOF | TokKind::Closer(Closer::RCurly) => {
    //                 let _ = self.take();
    //                 return Ok(children);
    //             }
    //             TokKind::Ident => match self.look_ahead()?.kind {
    //                 TokKind::Symbol(Symbol::Colon)
    //                 | TokKind::Symbol(Symbol::Dot) => children.push(Ast::File(self.parse_file_lit()?)),
    //                 TokKind::Opener(Opener::LCurly)
    //                 | TokKind::Symbol(Symbol::Comma) => children.push(Ast::Dir(self.parse_dir()?)),
    //                 TokKind::Closer(Closer::RCurly) => {
    //                     let name = self.take()?;
    //                     children.push(Ast::Dir(Dir {
    //                         params: Vec::new(),
    //                         name: std::path::PathBuf::from(name.val.unwrap()),
    //                         children: Vec::new(),
    //                         alias: name.val.unwrap().to_owned(),
    //                     }));
    //                     return Ok(children);
    //                 }
    //                 ki => {
    //                     println!("Unexpected token: {}", ki);
    //                     return Err(Trace::new_syn(next, "`:`, `.` for file or `{`, `}`, `,` for directory definition"));
    //                 }
    //             },
    //             TokKind::Symbol(Symbol::At) => {
    //                 self.take()?;
    //                 let finame = self.parse_file_ref()?;
    //                 let args = self.parse_args()?;
    //                 let _ = self.assert_union(&[
    //                     TokKind::Symbol(Symbol::Comma),
    //                     TokKind::Closer(Closer::RCurly),
    //                 ]);
    //                 children.push(Ast::Ref(Ref {
    //                     kind: AstKind::File,
    //                     name: finame,
    //                     args,
    //             }));
    //             }
    //             TokKind::Symbol(Symbol::Dollar) => {
    //                 self.take()?;
    //                 let dirname = self.assert(TokKind::Ident)?.val.unwrap().to_owned();
    //                 let args = self.parse_args()?;
    //                 let _ = self.assert_union(&[
    //                     TokKind::Symbol(Symbol::Comma),
    //                     TokKind::Closer(Closer::RCurly),
    //                 ]);
    //                 children.push(Ast::Ref(Ref {
    //                     kind: AstKind::Dir,
    //                     name: dirname,
    //                     args,
    //             }));
    //             }
    //             _ => return Err(Trace::new_syn(next, "struct- or file name")),

    //         }
    //     }
    // }

    pub fn parse_file_ref(&mut self) -> Result<String, Trace<'a, SynErr>> {
        let name = self.assert(TokKind::Ident)?;
        let mut buf = name.val.unwrap().to_owned();
        while let Ok(Token {
            kind: TokKind::Symbol(Symbol::Dot),
            ..
        }) = self.assert_union(&[
            TokKind::Symbol(Symbol::Dot),
            TokKind::Symbol(Symbol::Comma),
            TokKind::Closer(Closer::RCurly),
        ]) {
            self.take()?;
            let ext = self.assert(TokKind::Ident)?;
            buf = buf + "." + &ext.val.unwrap();
        }
        Ok(buf)
    }

    pub fn parse_file_lit(&mut self) -> Result<File, Trace<'a, SynErr>> {
        let (ident, mut path) =
            match self.assert_union(&[TokKind::Opener(Opener::DQuote), TokKind::Ident])? {
                Token {
                    kind: TokKind::Opener(Opener::DQuote),
                    ..
                } => {
                    let v = self
                        .assert(TokKind::Literal(Literal::String))?
                        .val
                        .unwrap()
                        .to_owned();
                    let _ = self.assert(TokKind::Closer(Closer::DQuote));
                    (v.clone(), Some(v))
                }
                Token {
                    kind: TokKind::Ident,
                    val,
                    ..
                } => {
                    let mut buf = val.unwrap().to_owned();
                    while let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Dot)) {
                        buf.push('.');
                        buf.push_str(self.assert(TokKind::Ident)?.val.unwrap());
                    }
                    (buf, None)
                }
                _ => unreachable!(),
            };

        let params = self.parse_params()?;
        if path.is_none() {
            path = if let Ok(Token {
                kind: TokKind::Symbol(Symbol::Colon),
                ..
            }) = self.look_ahead_one()
            {
                let _ = self.take()?;
                Some(self.parse_raw_string()?)
            } else {
                Some(ident.clone())
            }
        };

        let fi = File {
            name: path.unwrap(),
            alias: ident,
            content: self.parse_string()?,
            params,
        };

        self.assert_union(&[
            TokKind::Symbol(Symbol::Semi),
            TokKind::Symbol(Symbol::Comma),
            TokKind::Closer(Closer::RCurly),
            TokKind::EOF,
        ])?;

        Ok(fi)
    }

    pub fn parse_let(&mut self) -> Result<Let, Trace<'a, SynErr>> {
        let name = self.assert(TokKind::Ident)?;
        let params = self.parse_params()?;
        let _ = self.assert(TokKind::Symbol(Symbol::Equal))?;
        let expr = self.parse_string()?;
        let _ = self.assert_union(&[TokKind::Symbol(Symbol::Semi), TokKind::EOF])?;

        Ok(Let {
            name: name.val.unwrap().to_owned(),
            params,
            ty: Ty::Str,
            expr,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Ty)>, Trace<'a, SynErr>> {
        if let TokKind::Opener(Opener::LParen) = self.look_ahead_one()?.kind {
            let _ = self.assert(TokKind::Opener(Opener::LParen))?;
            let mut params = Vec::new();
            while let Ok(Token { kind, val, .. }) = self.assert_union(&[
                TokKind::Ident,
                TokKind::Symbol(Symbol::Comma),
                TokKind::Closer(Closer::RParen),
            ]) {
                match kind {
                    TokKind::Ident => {
                        self.assert(TokKind::Symbol(Symbol::Colon))?;
                        let ty: Ty = self.assert(TokKind::Ident)?.val.unwrap().into();
                        params.push((val.unwrap().to_owned(), ty));
                    }
                    TokKind::Closer(Closer::RParen) => break,
                    _ => {}
                }
            }
            Ok(params)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_args(&mut self) -> Result<Vec<(String, Expr)>, Trace<'a, SynErr>> {
        if let TokKind::Opener(Opener::LParen) = self.look_ahead_one()?.kind {
            self.take()?;
            let mut args = Vec::new();
            while let Ok(tok) = self.assert_union(&[
                TokKind::Ident,
                TokKind::Symbol(Symbol::Comma),
                TokKind::Closer(Closer::RParen),
            ]) {
                match tok.kind {
                    TokKind::Ident => {
                        self.assert(TokKind::Symbol(Symbol::Colon))?;
                        let arg_val = self
                            .assert_union(&[TokKind::Ident, TokKind::Literal(Literal::String)])?;
                        match arg_val.kind {
                            TokKind::Ident => {
                                let arg_args = self.parse_args()?;
                                args.push((
                                    tok.val.unwrap().to_owned(),
                                    Expr::Ref(Ref {
                                        kind: AstKind::Let,
                                        name: arg_val.val.unwrap().to_owned(),
                                        args: arg_args,
                                    }),
                                ));
                            }
                            TokKind::Literal(Literal::String) => {
                                args.push((
                                    tok.val.unwrap().to_owned(),
                                    Expr::Lit(Lit::String(arg_val.val.unwrap().to_owned())),
                                ));
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokKind::Closer(Closer::RParen) => break,
                    _ => {}
                }
            }
            self.assert(TokKind::Symbol(Symbol::Equal))?;
            Ok(args)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_string(&mut self) -> Result<Vec<Expr>, Trace<'a, SynErr>> {
        let mut buf = Vec::new();
        let mut openers = Stack::<16, Opener>::new();
        while let Ok(tok) = self.consume_if_union(&[
            TokKind::Opener(Opener::LCurlyDQuote),
            TokKind::Opener(Opener::LCurlyDollar),
            TokKind::Opener(Opener::DQuote),
            TokKind::Closer(Closer::RCurlyDQuote),
            TokKind::Closer(Closer::RCurlyDollar),
            TokKind::Closer(Closer::DQuote),
        ]) {
            println!("KIND: {}", tok.kind);
            match tok.kind {
                TokKind::Opener(Opener::LCurlyDollar) => {
                    let ident = self.assert(TokKind::Ident)?;
                    let args = self.parse_args()?;
                    openers.push(Opener::LCurlyDollar);
                    buf.push(Expr::Ref(Ref {
                        kind: AstKind::Let,
                        name: ident.val.unwrap().to_owned(),
                        args,
                    }));
                }
                TokKind::Opener(Opener::LCurlyDQuote) | TokKind::Opener(Opener::DQuote) => {
                    let string = self.assert(TokKind::Literal(Literal::String))?;
                    openers.push(tok.try_into().unwrap());
                    buf.push(Expr::Lit(Lit::String(string.val.unwrap().to_owned())));
                }
                TokKind::Closer(Closer::RCurlyDQuote)
                | TokKind::Closer(Closer::RCurlyDollar)
                | TokKind::Closer(Closer::DQuote) => {
                    if let Ok(op) = openers.peek() {
                        println!("OP:     {:?}", op);
                        println!("CLOSER: {:?}", Closer::try_from(tok));
                        if op.closer() == tok.try_into().unwrap() {
                            let _ = openers.pop();
                        } else {
                            return Err(Trace::new_syn(tok, "unmatched closer"));
                        }
                    } else {
                        return Err(Trace::new_syn(tok, "unmatched closer"));
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(buf)
    }

    fn parse_raw_string(&mut self) -> Result<String, Trace<'a, SynErr>> {
        let mut buf = String::new();
        if let Ok(tok) = self.assert(TokKind::Opener(Opener::DQuote)) {
            if let TokKind::Opener(Opener::DQuote) = tok.kind {
                let string = self.assert(TokKind::Literal(Literal::String))?;
                buf.push_str(&string.val.unwrap());
                let _ = self.assert(TokKind::Closer(Closer::DQuote));
            } else {
                unreachable!()
            }
        }
        Ok(buf)
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

    fn assert_union(&mut self, kinds: &[TokKind]) -> Result<Token<'a>, Trace<'a, SynErr>> {
        let tok = self.lx.next_token()?;
        if !kinds.contains(&tok.kind) {
            return Err(Trace::new(
                tok.src,
                SynErr::Expected(
                    kinds
                        .iter()
                        .map(|k| k.to_string())
                        .collect::<Vec<String>>()
                        .join(" or "),
                    tok.kind.to_string(),
                    tok.src.to_string(),
                ),
            ));
        }
        Ok(tok)
    }

    // fn assert_look_ahead(&mut self, kind: TokKind) -> Result<Token<'a>, Trace<'a, SynErr>> {
    //     let tok = self.lx.look_ahead_one()?;
    //     if tok.kind != kind {
    //         return Err(Trace::new(
    //             tok.src,
    //             SynErr::Expected(kind.to_string(), tok.kind.to_string(), tok.src.to_string()),
    //         ));
    //     }
    //     Ok(tok)
    // }

    // fn assert_union_look_ahead(
    //     &mut self,
    //     kinds: &[TokKind],
    // ) -> Result<Token<'a>, Trace<'a, SynErr>> {
    //     let tok = self.lx.look_ahead_one()?;
    //     if !kinds.contains(&tok.kind) {
    //         return Err(Trace::new(
    //             tok.src,
    //             SynErr::Expected(
    //                 kinds
    //                     .iter()
    //                     .map(|k| k.to_string())
    //                     .collect::<Vec<String>>()
    //                     .join(" or "),
    //                 tok.kind.to_string(),
    //                 tok.src.to_string(),
    //             ),
    //         ));
    //     }
    //     Ok(tok)
    // }

    fn consume_if(&mut self, kind: TokKind) -> Result<Token<'a>, Trace<'a, SynErr>> {
        let tok = self.lx.look_ahead_one()?;
        if tok.kind == kind {
            Ok(self.lx.next_token()?)
        } else {
            Err(Trace::new(
                tok.src,
                SynErr::Expected(kind.to_string(), tok.kind.to_string(), tok.src.to_string()),
            ))
        }
    }

    fn consume_if_union(&mut self, kinds: &[TokKind]) -> Result<Token<'a>, Trace<'a, SynErr>> {
        let tok = self.lx.look_ahead_one()?;
        if kinds.contains(&tok.kind) {
            Ok(self.lx.next_token()?)
        } else {
            Err(Trace::new(
                tok.src,
                SynErr::Expected(
                    kinds
                        .iter()
                        .map(|k| k.to_string())
                        .collect::<Vec<String>>()
                        .join(" or "),
                    tok.kind.to_string(),
                    tok.src.to_string(),
                ),
            ))
        }
    }
}

#[cfg(test)]
mod test {

    macro_rules! file_test {
        ($fn_name:ident, $src:expr, $name:expr, $alias:expr, $params:expr, $content:expr) => {
            #[test]
            fn $fn_name() {
                println!("TEST: {}", $src);
                let mut syn = crate::syntax::Syntax::new($src.as_bytes());
                let ast = syn.parse().expect("Failed to parse");
                assert_eq!(ast.name(), Some($name.into()));
                assert_eq!(ast.ty(), crate::ast::Ty::File);
                assert_eq!(ast.reqs(), $params);
                assert_eq!(ast.kind(), crate::ast::AstKind::File);
                if let crate::ast::Ast::File(f) = ast {
                    assert_eq!(f.name, $name);
                    assert_eq!(f.alias, $alias);
                    assert_eq!(f.params, $params);
                    assert_eq!(f.content, $content);
                }
            }
        };
    }

    // macro_rules! dir_test {
    //     ($fn_name:ident, $src:expr, $name:expr, $alias:expr, $params:expr, $children:expr) => {
    //         #[test]
    //         fn $fn_name() {
    //             println!("TEST: {}", $src);
    //             let mut syn = crate::syntax::Syntax::new($src.as_bytes());
    //             let ast = syn.parse().expect("Failed to parse");
    //             assert_eq!(ast.name(), Some($name.into()));
    //             assert_eq!(ast.ty(), crate::ast::Ty::Dir);
    //             assert_eq!(ast.reqs(), $params);
    //             assert_eq!(ast.kind(), crate::ast::AstKind::Dir);
    //             if let crate::ast::Ast::Dir(f) = ast {
    //                 assert_eq!(f.name, $name);
    //                 assert_eq!(f.alias, $alias);
    //                 assert_eq!(f.params, $params);
    //                 assert_eq!(f.children, $children);
    //             }
    //         }
    //     };
    // }

    file_test!(
        test_file,
        r#"file test: "test.txt" {"hello world"};"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(crate::ast::Lit::String(
            "hello world".into()
        ))]
    );
    file_test!(
        test_file_raw,
        r#"file test: "test.txt" "hello world";"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(crate::ast::Lit::String(
            "hello world".into()
        ))]
    );
    file_test!(
        test_file_alias,
        r#"file test: "test.txt" {"hello world"};"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(crate::ast::Lit::String(
            "hello world".into()
        ))]
    );
    file_test!(
        test_file_alias_raw,
        r#"file test: "test.txt" "hello world";"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(crate::ast::Lit::String(
            "hello world".into()
        ))]
    );

    file_test!(
        test_file_params,
        r#"file foo(bar: str): "voo.a" {"Hello {$bar$}"};"#,
        "voo.a",
        "foo",
        vec![("bar".into(), crate::ast::Ty::Str)],
        vec![
            crate::ast::Expr::Lit(crate::ast::Lit::String("Hello ".into())),
            crate::ast::Expr::Ref(crate::ast::Ref {
                kind: crate::ast::AstKind::Let,
                name: "bar".into(),
                args: Vec::new(),
            })
        ]
    );

    file_test!(
        test_file_params_no_str,
        r#"file foo(bar: str): "voo.a" {$bar$};"#,
        "voo.a",
        "foo",
        vec![("bar".into(), crate::ast::Ty::Str)],
        vec![crate::ast::Expr::Ref(crate::ast::Ref {
            kind: crate::ast::AstKind::Let,
            name: "bar".into(),
            args: Vec::new(),
        })]
    );

    file_test!(
        test_file_params_raw,
        r#"file "$foo.a"(bar: str) {$bar$};"#,
        "$foo.a",
        "$foo.a",
        vec![("bar".into(), crate::ast::Ty::Str)],
        vec![crate::ast::Expr::Ref(crate::ast::Ref {
            kind: crate::ast::AstKind::Let,
            name: "bar".into(),
            args: Vec::new(),
        })]
    );

    #[test]
    fn test_dir() {
        let src = r#"
            dir foo: foo/bar/baz;
            dir bar { #foo }
        "#;
        println!("TEST: {}", src);
        let mut syn = crate::syntax::Syntax::new(src.as_bytes());
        let ast = syn.parse_all().expect("Failed to parse");
        println!("{:#?}", ast);
        // assert_eq!(ast.name(), Some("foo/bar/baz".into()));
        // assert_eq!(ast.ty(), crate::ast::Ty::Dir);
        // assert_eq!(ast.reqs(), Vec::new());
        // assert_eq!(ast.kind(), crate::ast::AstKind::Dir);
        // if let crate::ast::Ast::Dir(f) = ast {
        //     assert_eq!(f.name, std::path::PathBuf::from("foo/bar/baz"));
        //     assert_eq!(f.alias, "foo");
        //     assert_eq!(f.params, Vec::new());
        //     assert_eq!(
        //         f.children,
        //         vec![crate::ast::Ast::Dir(crate::ast::Dir {
        //             name: std::path::PathBuf::from("foo/bar/baz"),
        //             alias: "bar".into(),
        //             params: Vec::new(),
        //             children: Vec::new(),
        //         })]
        //     );
        // }
    }
}
