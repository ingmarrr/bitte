use crate::{
    ast::{Ast, Dir, Expr, File, Let, Lit, Ref, Ty},
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
        Ok(self.lx.look_ahead()?)
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
            TokKind::Keyword(Keyword::Main) => {
                let mut ast = self.parse()?;
                ast.set_main();
                Ok(ast)
            }
            TokKind::Keyword(Keyword::Dir) => Ok(Ast::Dir(self.parse_dir_lit()?)),
            TokKind::Keyword(Keyword::Let) => Ok(Ast::Let(self.parse_let()?)),
            TokKind::Keyword(Keyword::File) => Ok(Ast::File(self.parse_file_lit()?)),
            TokKind::Keyword(Keyword::Req) => Ok(Ast::Ref(self.parse_req()?)),

            _ => {
                let tok = self.lx.look_ahead()?;
                Err(Trace::new(
                    tok.src,
                    SynErr::Expected(
                        "struct or file".into(),
                        tok.kind.to_string(),
                        tok.src.to_string(),
                    ),
                ))
            }
        }
    }

    pub fn parse_dir_lit(&mut self) -> Result<Dir, Trace<'a, SynErr>> {
        let (ident, mut path) = {
            let tok = self.assert_union(&[TokKind::Opener(Opener::DQuote), TokKind::Ident])?;
            match tok.kind {
                TokKind::Opener(Opener::DQuote) => {
                    let v = self.assert(TokKind::Literal(Literal::String))?.val_owned();
                    let _ = self.assert(TokKind::Closer(Closer::DQuote));
                    (v.clone(), Some(v))
                }
                TokKind::Ident => {
                    let mut buf = tok.val_owned();
                    while let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Slash)) {
                        buf.push('/');
                        buf.push_str(self.assert(TokKind::Ident)?.val());
                    }
                    (buf, None)
                }
                _ => unreachable!(),
            }
        };

        let params = self.parse_params()?;
        if path.is_none() {
            path = if let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Colon)) {
                if let TokKind::Opener(Opener::DQuote) = self.look_ahead_one()?.kind {
                    Some(self.parse_raw_string()?)
                } else {
                    let mut buf = self.assert(TokKind::Ident)?.val().to_owned();
                    while let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Slash)) {
                        buf.push('/');
                        buf.push_str(self.assert(TokKind::Ident)?.val());
                    }
                    Some(buf)
                }
            } else {
                Some(ident.clone())
            }
        };

        let di = Dir {
            main: false,
            path: std::path::PathBuf::from(path.unwrap()),
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
            TokKind::Symbol(Symbol::At),
            TokKind::Symbol(Symbol::Comma),
            TokKind::Opener(Opener::DQuote),
            TokKind::Ident,
        ]) {
            println!("TOK: {:?}", tok);
            match tok.kind {
                TokKind::Symbol(Symbol::Comma) => {}
                TokKind::Symbol(Symbol::At) => {
                    children.push(Ast::Ref(Ref {
                        name: self.assert(TokKind::Ident)?.val_owned(),
                        args: self.parse_args()?,
                        ty: Ty::Unknown,
                    }));
                    if let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Comma)) {
                        if let Ok(_) = self.consume_if(TokKind::Closer(Closer::RCurly)) {
                            break;
                        }
                        continue;
                    }
                    self.assert(TokKind::Closer(Closer::RCurly))?;
                }
                TokKind::Opener(Opener::DQuote) => {
                    let name = self.assert(TokKind::Literal(Literal::String))?.val();
                    let _ = self.assert(TokKind::Closer(Closer::DQuote))?;
                    if let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Colon)) {
                        let content = if let Ok(ident) = self.consume_if(TokKind::Ident) {
                            vec![Expr::Ref(Ref {
                                name: ident.val_owned(),
                                args: self.parse_args()?,
                                ty: Ty::String,
                            })]
                        } else {
                            self.parse_string()?
                        };
                        children.push(Ast::File(File {
                            main: false,
                            params: Vec::new(),
                            path: name.to_owned(),
                            alias: name.to_owned(),
                            content,
                        }))
                    } else {
                        children.push(Ast::File(File {
                            main: false,
                            params: Vec::new(),
                            path: name.to_owned(),
                            alias: name.to_owned(),
                            content: Vec::new(),
                        }))
                    }
                    self.assert_union(&[
                        TokKind::Symbol(Symbol::Comma),
                        TokKind::Closer(Closer::RCurly),
                    ])?;
                }
                TokKind::Ident => {
                    if let TokKind::Opener(Opener::LCurly) = self.look_ahead_one()?.kind {
                        children.push(Ast::Dir(Dir {
                            main: false,
                            params: Vec::new(),
                            path: tok.val_owned().into(),
                            alias: tok.val_owned(),
                            children: self.parse_children()?,
                        }));
                        let _ = self.consume_if_union(&[TokKind::Closer(Closer::RCurly)]);
                        continue;
                    }

                    let next = self.assert_union(&[
                        TokKind::Symbol(Symbol::Comma),
                        TokKind::Symbol(Symbol::Colon),
                        TokKind::Closer(Closer::RCurly),
                    ])?;

                    match next.kind {
                        TokKind::Symbol(Symbol::Comma) => {
                            children.push(Ast::Dir(Dir {
                                main: false,
                                params: Vec::new(),
                                path: tok.val_owned().into(),
                                alias: tok.val_owned(),
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
                                    main: false,
                                    params: Vec::new(),
                                    path: tok.val_owned().into(),
                                    alias: tok.val_owned(),
                                    content: vec![Expr::Ref(Ref {
                                        name: self.assert(TokKind::Ident)?.val_owned(),
                                        args: self.parse_args()?,
                                        ty: Ty::String,
                                    })],
                                }))
                            } else {
                                children.push(Ast::File(File {
                                    main: false,
                                    params: Vec::new(),
                                    path: tok.val_owned().into(),
                                    alias: tok.val_owned(),
                                    content: self.parse_string()?,
                                }))
                            }
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

    pub fn parse_file_lit(&mut self) -> Result<File, Trace<'a, SynErr>> {
        let (ident, mut path) = {
            let tok = self.assert_union(&[TokKind::Opener(Opener::DQuote), TokKind::Ident])?;
            match tok.kind {
                TokKind::Opener(Opener::DQuote) => {
                    let v = self.assert(TokKind::Literal(Literal::String))?.val_owned();
                    let _ = self.assert(TokKind::Closer(Closer::DQuote));
                    (v.clone(), Some(v))
                }
                TokKind::Ident => {
                    let mut buf = tok.val_owned();
                    while let Ok(_) = self.consume_if(TokKind::Symbol(Symbol::Dot)) {
                        buf.push('.');
                        buf.push_str(self.assert(TokKind::Ident)?.val());
                    }
                    (buf, None)
                }
                _ => unreachable!(),
            }
        };

        let params = self.parse_params()?;
        if path.is_none() {
            path = if let Ok(Token {
                kind: TokKind::Symbol(Symbol::Colon),
                ..
            }) = self.look_ahead_one()
            {
                let _ = self.take();
                Some(self.parse_raw_string()?)
            } else {
                Some(ident.clone())
            }
        };

        let fi = File {
            main: false,
            path: path.unwrap(),
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
            main: false,
            name: name.val_owned(),
            params,
            ty: Ty::String,
            expr,
        })
    }

    pub fn parse_req(&mut self) -> Result<Ref, Trace<'a, SynErr>> {
        let name = self.assert(TokKind::Ident)?;
        let _ = self.assert(TokKind::Symbol(Symbol::Colon))?;
        let ty: Ty = self.assert(TokKind::Ident)?.val().into();
        let _ = self.assert(TokKind::Symbol(Symbol::Semi))?;

        Ok(Ref {
            name: name.val_owned(),
            args: vec![],
            ty,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Ty)>, Trace<'a, SynErr>> {
        if let TokKind::Opener(Opener::LParen) = self.look_ahead_one()?.kind {
            let _ = self.assert(TokKind::Opener(Opener::LParen))?;
            let mut params = Vec::new();
            while let Ok(tok) = self.assert_union(&[
                TokKind::Ident,
                TokKind::Symbol(Symbol::Comma),
                TokKind::Closer(Closer::RParen),
            ]) {
                match tok.kind {
                    TokKind::Ident => {
                        self.assert(TokKind::Symbol(Symbol::Colon))?;
                        let ty: Ty = self.assert(TokKind::Ident)?.val().into();
                        params.push((tok.val_owned(), ty));
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
                        let arg_val =
                            self.assert_union(&[TokKind::Ident, TokKind::Opener(Opener::DQuote)])?;
                        match arg_val.kind {
                            TokKind::Ident => {
                                let arg_args = self.parse_args()?;
                                args.push((
                                    tok.val_owned(),
                                    Expr::Ref(Ref {
                                        name: arg_val.val_owned(),
                                        args: arg_args,
                                        ty: Ty::String,
                                    }),
                                ));
                            }
                            TokKind::Opener(Opener::DQuote) => {
                                let arg_string = self.assert(TokKind::Literal(Literal::String))?;
                                let _ = self.assert(TokKind::Closer(Closer::DQuote));
                                args.push((
                                    tok.val_owned(),
                                    Expr::Lit(Lit::String(arg_string.val_owned())),
                                ));
                            }
                            _ => unreachable!(),
                        }
                    }
                    TokKind::Closer(Closer::RParen) => break,
                    _ => {}
                }
            }
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
            match tok.kind {
                TokKind::Opener(Opener::LCurlyDollar) => {
                    let ident = self.assert(TokKind::Ident)?;
                    let args = self.parse_args()?;
                    openers.push(Opener::LCurlyDollar);
                    buf.push(Expr::Ref(Ref {
                        name: ident.val_owned(),
                        args,
                        ty: Ty::String,
                    }));
                }
                TokKind::Opener(Opener::LCurlyDQuote) | TokKind::Opener(Opener::DQuote) => {
                    let string = self.assert(TokKind::Literal(Literal::String))?;
                    openers.push(tok.try_into().unwrap());
                    buf.push(Expr::Lit(Lit::String(string.val_owned())));
                }
                TokKind::Closer(Closer::RCurlyDollar) => {
                    if let Ok(op) = openers.peek() {
                        if op.closer() == Closer::RCurlyDollar {
                            let _ = openers.pop();
                            if let Ok(Opener::LCurlyDQuote) = openers.peek() {
                                let string = self.lx.try_lx_str()?;
                                buf.push(Expr::Lit(Lit::String(string.val_owned())));
                            }
                        } else {
                            return Err(Trace::new_syn(tok, "unmatched opener: {{\""));
                        }
                    } else {
                        return Err(Trace::new_syn(tok, "unmatched closer: $}"));
                    }
                }
                TokKind::Closer(Closer::RCurlyDQuote) | TokKind::Closer(Closer::DQuote) => {
                    if let Ok(op) = openers.peek() {
                        if op.closer() == tok.try_into().unwrap() {
                            let _ = openers.pop();
                        } else {
                            return Err(Trace::new_syn(
                                tok,
                                &("unmatched closer".to_owned() + &op.to_string()),
                            ));
                        }
                    } else {
                        return Err(Trace::new_syn(
                            tok,
                            &("unmatched closer".to_owned() + &tok.to_string()),
                        ));
                    }
                    if openers.is_empty() {
                        break;
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
                buf.push_str(&string.val());
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

    fn consume_if(&mut self, kind: TokKind) -> Result<Token<'a>, Trace<'a, SynErr>> {
        let tok = self.lx.look_ahead()?;
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
        let tok = self.lx.look_ahead()?;
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
    use crate::ast::{Lit, Ty};

    macro_rules! file_test {
        ($fn_name:ident, $src:expr, $name:expr, $alias:expr, $params:expr, $content:expr) => {
            #[test]
            fn $fn_name() {
                println!("TEST: {}", $src);
                let mut syn = crate::syntax::Syntax::new($src.as_bytes());
                let ast = syn.parse().expect("Failed to parse");
                // assert_eq!(ast.name(), $name.into());
                assert_eq!(ast.ty(), crate::ast::Ty::File);
                assert_eq!(ast.params(), $params);
                assert_eq!(ast.kind(), crate::ast::AstKind::File);
                if let crate::ast::Ast::File(f) = ast {
                    assert_eq!(f.path, $name);
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
        vec![crate::ast::Expr::Lit(Lit::String("hello world".into()))]
    );
    file_test!(
        test_file_raw,
        r#"file test: "test.txt" "hello world";"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(Lit::String("hello world".into()))]
    );
    file_test!(
        test_file_alias,
        r#"file test: "test.txt" {"hello world"};"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(Lit::String("hello world".into()))]
    );
    file_test!(
        test_file_alias_raw,
        r#"file test: "test.txt" "hello world";"#,
        "test.txt",
        "test",
        Vec::new(),
        vec![crate::ast::Expr::Lit(Lit::String("hello world".into()))]
    );

    file_test!(
        test_file_params,
        r#"file foo(bar: str): "voo.a" {"Hello {$bar$}"};"#,
        "voo.a",
        "foo",
        vec![("bar".into(), crate::ast::Ty::String)],
        vec![
            crate::ast::Expr::Lit(Lit::String("Hello ".into())),
            crate::ast::Expr::Ref(crate::ast::Ref {
                // kind: crate::ast::AstKind::Let,
                name: "bar".into(),
                args: Vec::new(),
                ty: Ty::String,
            }),
            crate::ast::Expr::Lit(Lit::String("".into())),
        ]
    );

    file_test!(
        test_file_params_no_str,
        r#"file foo(bar: str): "voo.a" {$bar$};"#,
        "voo.a",
        "foo",
        vec![("bar".into(), crate::ast::Ty::String)],
        vec![crate::ast::Expr::Ref(crate::ast::Ref {
            // kind: crate::ast::AstKind::Let,
            name: "bar".into(),
            args: Vec::new(),
            ty: Ty::String,
        })]
    );

    file_test!(
        test_file_params_raw,
        r#"file "$foo.a"(bar: str) {$bar$};"#,
        "$foo.a",
        "$foo.a",
        vec![("bar".into(), crate::ast::Ty::String)],
        vec![crate::ast::Expr::Ref(crate::ast::Ref {
            // kind: crate::ast::AstKind::Let,
            name: "bar".into(),
            args: Vec::new(),
            ty: Ty::String,
        })]
    );

    #[test]
    fn test_dir() {
        let src = r#"
            dir foo: foo/bar/baz;
            dir bar { @foo }
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

    #[test]
    fn test_dir_name_ref() {
        let src = r#"
            let name = "hello";
            dir bar: @name { #foo }
        "#;

        println!("TEST: {}", src);
        // let mut syn = crate::syntax::Syntax::new(src.as_bytes());
        // let ast = syn.parse_all().expect("Failed to parse");
    }

    #[test]
    fn test_file_must_compile_all() {
        let src = r##"
            file test;
            file test.md;
            file "test.md";
            file test: "test.md";
            file test "# Test";
            file test {"# Test"};
            file "test.md" "# Test";
            file "test.md" {"# Test"};
            file test: "test.md" "# Test";
            file test: "test.md" {"# Test"};
            file test();
            file "test.md"();
            file test(): "test.md";
            file test(): "test.md" "# Test";
            file test(): "test.md" {"# Test"};
            file test(bar: str);
            file "test.md"(bar: str);
            file test(bar: str): "test.md";
            file test(bar: str): "test.md" "# Test";
            file test(bar: str): "test.md" {"# Test"};
            file test(bar: str): "test.md" {$bar$};
            file test(bar: str): "test.md" {"Test {$bar$}"};
            file test.md(bar: str) {"Test {$bar$}"};
            file "test.md"(bar: str) {$bar$};
            file test.md(bar: str) {"Test {$bar$}{$bar$}"};
            main file test;
        "##;

        println!("TEST: {}", src);
        let mut syn = crate::syntax::Syntax::new(src.as_bytes());
        let ast = syn.parse_all();
        assert!(ast.is_ok());
    }

    #[test]
    fn test_dir_must_compile_all() {
        let src = r##"
            dir test;
            dir test: "test/foo/bar";
            dir foo/bar/baz;
        "##;

        println!("TEST: {}", src);
        let mut syn = crate::syntax::Syntax::new(src.as_bytes());
        let ast = syn.parse_all();
        assert!(ast.is_ok());
    }
}
