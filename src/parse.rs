use crate::{
    ast::{Stmt, Ty},
    decl::{Decl, Fmt, Let, Struct},
    err::ParseError,
    expr::{Expr, FmtCall, Insert, Inserted, List, Lit, StructCall},
    parse,
    token::{Tok, TokKind},
};
pub struct Parser {
    lx: crate::lex::Lexer,
}

impl Parser {
    pub fn new(lx: crate::lex::Lexer) -> Self {
        Self { lx }
    }

    pub fn parse(&mut self) -> Result<Vec<Decl>, ParseError> {
        let mut stmts = Vec::new();

        loop {
            stmts.push(match self.lx.look_ahead()? {
                // tok if tok.is_str() => Stmt::Expr(self.parse_str()?),
                Tok::Struct => self.parse_struct()?,
                Tok::Let => self.parse_let()?,
                Tok::Fmt => self.parse_fmt()?,
                Tok::Ident(_) => return Err(ParseError::NoTopLevelExpressionsAllowed),
                Tok::EOF => return Ok(stmts),
                _t => {
                    debug!("Parsing :: {:#?}", _t);
                    return Err(ParseError::Unimplemented);
                }
            });
        }
    }

    fn parse_fmt(&mut self) -> Result<Decl, ParseError> {
        self.lx.assert_next_token(TokKind::Fmt)?;
        let name = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };
        self.lx.assert_next_token(TokKind::LParen)?;
        let params = self.parse_params()?;
        self.lx.assert_next_token(TokKind::Eq)?;
        let body = self.parse_str()?;
        if let Expr::Inserted(Inserted {
            string_parts,
            inserts,
        }) = body
        {
            return Ok(Decl::Fmt(Fmt {
                name,
                params,
                string_parts,
                inserts,
            }));
        }
        if let Expr::Lit(Lit::Str(body)) = body {
            return Ok(Decl::Fmt(Fmt {
                name,
                params,
                string_parts: vec![body],
                inserts: vec![],
            }));
        }
        Err(ParseError::Expected {
            line: self.lx.cx.line,
            col: self.lx.cx.col,
            expected: "string".to_string(),
            found: "invalid expression".to_string(),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<(Box<str>, Ty)>, ParseError> {
        let mut fields = Vec::new();
        loop {
            let name = match self.lx.assert_next_token(TokKind::Ident)? {
                Tok::Ident(name) => name,
                _ => unreachable!(),
            };
            self.lx.assert_next_token(TokKind::Colon)?;
            let ty = self.parse_ty()?;
            fields.push((name, ty));
            match self.lx.look_ahead()? {
                Tok::Comma => {
                    self.lx.assert_next_token(TokKind::Comma)?;
                }
                Tok::RParen => {
                    self.lx.assert_next_token(TokKind::RParen)?;
                    return Ok(fields);
                }
                t => {
                    return Err(ParseError::Expected {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        expected: "comma or `)`".to_string(),
                        found: t.to_string(),
                    })
                }
            }
        }
    }

    fn parse_fields(&mut self) -> Result<Vec<(Box<str>, Expr)>, ParseError> {
        todo!()
    }

    fn parse_struct(&mut self) -> Result<Decl, ParseError> {
        let _ = self.lx.assert_next_token(TokKind::Struct)?;
        let name = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };
        let s = self.rec_parse_struct(name)?;

        Ok(Decl::Struct(s))
    }

    fn rec_parse_struct(&mut self, name: Box<str>) -> Result<Struct, ParseError> {
        parse!("{}", name);
        let mut children = Vec::new();
        let mut files = Vec::new();
        let mut params = Vec::new();

        match self.lx.look_ahead()? {
            Tok::Semi | Tok::RBrace | Tok::Comma => {
                parse!("Found `;` or `}}`");
                return Ok(Struct {
                    name,
                    children,
                    files,
                    params,
                });
            }
            Tok::LParen => {
                parse!("Found `(`");
                self.lx.assert_next_token(TokKind::LParen)?;
                params = self.parse_params()?;
                self.lx.assert_next_token(TokKind::RParen)?;
            }
            _ => {}
        };
        match self.lx.next_token()? {
            Tok::LBrace => loop {
                parse!("Parsing struct :: {:?}", self.lx.look_ahead()?);
                match self.lx.next_token()? {
                    Tok::RBrace => {
                        parse!("Found `}}`");
                        return Ok(Struct {
                            name,
                            children,
                            files,
                            params,
                        });
                    }
                    Tok::FilePath(path) => {
                        parse!("Found file path :: {}", path);
                        match self.lx.look_ahead()? {
                            Tok::Colon => {
                                self.lx.assert_next_token(TokKind::Colon)?;
                                let content = self.parse_expr()?;
                                files.push((path, Some(content)));
                            }
                            Tok::Comma | Tok::RBrace => files.push((path, None)),
                            _ => {
                                return Err(ParseError::Expected {
                                    line: self.lx.cx.line,
                                    col: self.lx.cx.col,
                                    expected: "`:`, `,` or `}}`".to_string(),
                                    found: self.lx.look_ahead()?.to_string(),
                                })
                            }
                        }
                    }
                    Tok::Ident(n) => children.push(self.rec_parse_struct(n)?),
                    Tok::Comma => {}
                    tok => {
                        parse!("Error Parsing :: {:?}", tok);
                        return Err(ParseError::Expected {
                            line: self.lx.cx.line,
                            col: self.lx.cx.col,
                            expected: "`{`, file path or struct name".to_string(),
                            found: tok.to_string(),
                        });
                    }
                }
            },
            _ => Err(ParseError::Expected {
                line: self.lx.cx.line,
                col: self.lx.cx.col,
                expected: "struct opening/ending or file ending".to_string(),
                found: self.lx.look_ahead()?.to_string(),
            }),
        }
    }

    fn parse_let(&mut self) -> Result<Decl, ParseError> {
        parse!("Parsing let");
        let _ = self.lx.assert_next_token(TokKind::Let)?;
        let name = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };

        let mut ty = None;
        if let Tok::Colon = self.lx.look_ahead()? {
            let _ = self.lx.assert_next_token(TokKind::Colon)?;
            ty = Some(self.parse_ty()?);
            parse!("FOUND TYPE :: {:?}", ty)
        }

        let _ = self.lx.assert_next_token(TokKind::Eq)?;
        parse!("FOUND `=`");
        let expr = self.parse_expr()?;
        if ty.is_none() {
            ty = Some(match &expr {
                Expr::Lit(Lit::Str(_)) => Ty::Str,
                Expr::Lit(Lit::Char(_)) => Ty::Char,
                Expr::Lit(Lit::List(_)) => Ty::List,
                Expr::Fmt { .. } | Expr::Inserted { .. } | Expr::Ident(_) => Ty::Str,
                _ => {
                    return Err(ParseError::Expected {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        expected: "type".to_string(),
                        found: "invalid expression".to_string(),
                    })
                }
            });
        }

        Ok(Decl::Let(Let {
            name,
            ty: ty.unwrap(),
            expr,
        }))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        parse!("Parsing expr");
        match self.lx.look_ahead()? {
            t if t.is_str() => self.parse_str(),
            Tok::LBracket => Ok(Expr::Lit(Lit::List(self.parse_list()?))),
            Tok::Ident(i) => match self.lx.look_ahead() {
                Ok(Tok::LParen) => Ok(Expr::Fmt(self.parse_fmt_call()?)),
                Ok(Tok::LBrace) => Ok(Expr::Struct(self.parse_struct_call()?)),
                Ok(_) => {
                    self.lx.next_token()?;
                    Ok(Expr::Ident(i))
                }
                Err(e) => Err(e.into()),
            },
            t => {
                parse!("Parsing expr :: {:#?}", t);
                todo!()
            }
        }
    }

    fn parse_fmt_call(&mut self) -> Result<FmtCall, ParseError> {
        let name = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };
        let _ = self.lx.assert_next_token(TokKind::LParen)?;
        let mut fields = Vec::new();
        loop {
            if let Tok::RParen = self.lx.look_ahead()? {
                let _ = self.lx.assert_next_token(TokKind::RParen)?;
                return Ok(FmtCall { name, fields });
            }
            let expr = self.parse_expr()?;
            fields.push((name.clone(), expr));
        }
    }

    fn parse_struct_call(&mut self) -> Result<StructCall, ParseError> {
        todo!()
    }

    fn parse_str(&mut self) -> Result<Expr, ParseError> {
        let mut ix = 0;
        let mut out_body = Vec::new();
        let mut inserts = Vec::new();

        loop {
            parse!("Parsing lit str :: {:?}", self.lx.look_ahead()?);
            match self.lx.next_token()? {
                Tok::String(body) => return Ok(Expr::Lit(Lit::Str(body))),
                Tok::DollarStarted(body) => {
                    out_body.push(body);
                    return Ok(Expr::Inserted(Inserted {
                        string_parts: out_body,
                        inserts,
                    }));
                }
                Tok::InBetween(body) | Tok::DollarTerminated(body) => {
                    ix += 1;
                    out_body.push(body);
                }
                Tok::Ident(i) => inserts.push((ix, self.parse_insertion(i)?)),
                t => {
                    return Err(ParseError::InvalidToken {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        tok: t.to_string(),
                    })
                }
            }
        }
    }

    fn parse_insertion(&mut self, ident: Box<str>) -> Result<Insert, ParseError> {
        // parse!("Parsing insertion, next token :: {:#?}", self.lx.look_ahead()?);
        match self.lx.look_ahead()? {
            Tok::For => {
                parse!("Found ident, starting for loop");
                return self.parse_for(Insert::Ident(ident));
            }
            Tok::LParen => {
                let _ = self.lx.assert_next_token(TokKind::LParen)?;
                let mut args = Vec::new();
                loop {
                    if let Tok::RParen = self.lx.look_ahead()? {
                        let _ = self.lx.assert_next_token(TokKind::RParen)?;
                        return match self.lx.look_ahead()? {
                            Tok::For => self.parse_for(Insert::Ident(ident)),
                            _ => Ok(Insert::Fmt { name: ident, args }),
                        };
                    }
                    args.push(self.parse_expr()?);
                }
            }
            _ => {
                parse!("found ident :: {}", ident);
                Ok(Insert::Ident(ident))
            }
        }
    }

    fn parse_for(&mut self, expr: Insert) -> Result<Insert, ParseError> {
        let _ = self.lx.assert_next_token(TokKind::For)?;

        let item = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(item) => item,
            _ => unreachable!(),
        };
        let _ = self.lx.assert_next_token(TokKind::In)?;
        let list = self.parse_list()?;
        let out = Ok(Insert::For {
            expr: Box::new(expr),
            item,
            iter: list,
        });
        parse!("Found for loop :: {:#?}", out);
        out
    }

    fn parse_list(&mut self) -> Result<List, ParseError> {
        let _ = self.lx.assert_next_token(TokKind::LBracket)?;
        let mut items = Vec::new();
        loop {
            if let Tok::RBracket = self.lx.look_ahead()? {
                let _ = self.lx.assert_next_token(TokKind::RBracket)?;
                return Ok(List::LitList(items));
            }
            match self.lx.next_token()? {
                Tok::String(item) => items.push(item),
                Tok::Comma => {}
                t => {
                    parse!("Parsing list :: {:#?}", t);
                    return Err(ParseError::Expected {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        expected: "string".to_string(),
                        found: t.to_string(),
                    });
                }
            }
        }
    }

    fn parse_ty(&mut self) -> Result<Ty, ParseError> {
        match self.lx.next_token()? {
            Tok::Ident(ty) => match &(*ty) {
                "str" => Ok(Ty::Str),
                "char" => Ok(Ty::Char),
                "int" => Ok(Ty::Int),
                "list" => Ok(Ty::List),
                _ => Ok(Ty::Struct),
            },
            t => Err(ParseError::Expected {
                line: self.lx.cx.line,
                col: self.lx.cx.col,
                expected: "type".to_string(),
                found: t.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::Lexer;
    use crate::test;

    macro_rules! ps_test {
           ($( $name:ident, $inp:expr, $expected:expr; )*) => {
               $(
                   #[test]
                   fn $name() {
                       test!("Testing `{}` = `{}`", stringify!($name), $inp);
                       parse!("Testing `{}` = `{}`", stringify!($name), $inp);
                       let lex = Lexer::new($inp);
                       let mut parser = Parser::new(lex);
                       let def = parser.parse();
                       assert_eq!(def, $expected);
                   }
               )*
           };
           ($name:ident, $inp:expr, $expected:expr) => {
               #[test]
               fn $name() {
                   test!(PAR, "Testing `{}` = `{}`", stringify!($name), $inp);
                   let lex = Lex::new($inp);
                   let mut parser = Parser::new(lex);
                   let def = parser.parse();
                   assert_eq!(def, $expected);
               }
           };
       }

    ps_test!(
        test_struct,
        "struct hello { world }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![Struct {
                name: "world".to_string().into_boxed_str(),
                children: vec![],
                files: vec![],
                params: vec![],
            }],
            files: vec![],
            params: vec![],
        })]);
        test_struct_nested,
        "struct hello { world { nested } }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![Struct {
                name: "world".to_string().into_boxed_str(),
                children: vec![Struct {
                    name: "nested".to_string().into_boxed_str(),
                    children: vec![],
                    files: vec![],
                    params: vec![],
                }],
                files: vec![],
                params: vec![],
            }],
            files: vec![],
            params: vec![],
        })]);
        test_struct_files,
        "struct hello { world.csv }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![],
            files: vec![("world.csv".to_string().into_boxed_str(), None)],
            params: vec![],
        })]);
        test_struct_files_nested,
        "struct hello { world { nested.csv } }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![Struct {
                name: "world".to_string().into_boxed_str(),
                children: vec![],
                files: vec![("nested.csv".to_string().into_boxed_str(), None)],
                params: vec![],
            }],
            files: vec![],
            params: vec![],
        })]);
        test_multiple_files,
        "struct hello { world.csv, nested.csv }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![],
            files: vec![
                ("world.csv".to_string().into_boxed_str(), None),
                ("nested.csv".to_string().into_boxed_str(), None)
            ],
            params: vec![],
        })]);
        test_multiple_nested_structs,
        "struct hello { world { world1.csv, world2.csv }, nested { nested1.csv, nested2.csv } }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![
                Struct {
                    name: "world".to_string().into_boxed_str(),
                    children: vec![],
                    files: vec![
                        ("world1.csv".to_string().into_boxed_str(), None),
                        ("world2.csv".to_string().into_boxed_str(), None),
                    ],
                    params: vec![],
                },
                Struct {
                    name: "nested".to_string().into_boxed_str(),
                    children: vec![],
                    files: vec![
                        ("nested1.csv".to_string().into_boxed_str(), None),
                        ("nested2.csv".to_string().into_boxed_str(), None),
                    ],
                    params: vec![],
                }
            ],
            files: vec![],
            params: vec![],
        })]);
        test_multiple_nested_structs2,
        "struct hello { world { world1.csv, anotherworld, world2.csv }, nested { nested1.csv, nested2.csv } }",
        Ok(vec![Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![
                Struct {
                    name: "world".to_string().into_boxed_str(),
                    children: vec![Struct {
                        name: "anotherworld".to_string().into_boxed_str(),
                        children: vec![],
                        files: vec![],
                        params: vec![],
                    }],
                    files: vec![
                        ("world1.csv".to_string().into_boxed_str(), None),
                        ("world2.csv".to_string().into_boxed_str(), None),
                    ],
                    params: vec![],
                },
                Struct {
                    name: "nested".to_string().into_boxed_str(),
                    children: vec![],
                    files: vec![
                        ("nested1.csv".to_string().into_boxed_str(), None),
                        ("nested2.csv".to_string().into_boxed_str(), None),
                    ],
                    params: vec![],
                }
            ],
            files: vec![],
            params: vec![],
        })]);
    );

    ps_test!(
        test_fmt,
        r#"fmt hello(name: str) = "hello, $name$""#,
        Ok(vec![Decl::Fmt(Fmt {
            name: "hello".to_string().into_boxed_str(),
            params: vec![("name".to_string().into_boxed_str(), Ty::Str)],
            string_parts: vec!["hello, ".to_string().into_boxed_str(), "".to_string().into_boxed_str()],
            inserts: vec![(1, Insert::Ident("name".to_string().into_boxed_str()))],
        })]);
        test_fmt_multiple_args,
        r#"fmt hello(name: str, age: int) = "hello, $name$, you are $age$""#,
        Ok(vec![Decl::Fmt(Fmt {
            name: "hello".to_string().into_boxed_str(),
            params: vec![
                ("name".to_string().into_boxed_str(), Ty::Str),
                ("age".to_string().into_boxed_str(), Ty::Int)
            ],
            string_parts: vec![
                "hello, ".to_string().into_boxed_str(),
                ", you are ".to_string().into_boxed_str(),
                "".to_string().into_boxed_str()
            ],
            inserts: vec![
                (1, Insert::Ident("name".to_string().into_boxed_str())),
                (2, Insert::Ident("age".to_string().into_boxed_str()))
            ],
        })]);
        test_fmt_for,
        r#"fmt hello(greet: str) = "hello, $greet for name in ["world", "bob"]$""#,
        Ok(vec![Decl::Fmt(Fmt {
            name: "hello".to_string().into_boxed_str(),
            params: vec![("greet".to_string().into_boxed_str(), Ty::Str)],
            string_parts: vec!["hello, ".to_string().into_boxed_str(), "".to_string().into_boxed_str()],
            inserts: vec![(1, Insert::For {
                expr: Box::new(Insert::Ident("greet".to_string().into_boxed_str())),
                item: "name".to_string().into_boxed_str(),
                iter: List::LitList(vec!["world".to_string().into_boxed_str(), "bob".to_string().into_boxed_str()]),
            })],
        })]);
    );

    ps_test!(
        multiple_decls,
        r#"struct hello { world.csv } struct world { hello.csv }"#,
        Ok(vec![
            Decl::Struct(Struct {
                name: "hello".to_string().into_boxed_str(),
                children: vec![],
                files: vec![("world.csv".to_string().into_boxed_str(), None)],
                params: vec![],
            }),
            Decl::Struct(Struct {
                name: "world".to_string().into_boxed_str(),
                children: vec![],
                files: vec![("hello.csv".to_string().into_boxed_str(), None)],
                params: vec![],
            })
        ]);
    );

    #[test]
    fn test_let() {
        let inp = "let a = \"hello\"";
        let lex = Lexer::new(inp);
        let mut par = Parser::new(lex);
        let out = par.parse();
        let exp = Ok(vec![Decl::Let(Let {
            name: "a".to_string().into_boxed_str(),
            ty: Ty::Str,
            expr: Expr::Lit(Lit::Str("hello".to_owned().into_boxed_str())),
        })]);
        assert_eq!(out, exp);
    }
}
