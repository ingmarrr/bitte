use crate::{
    ast::{Decl, Expr, Insert, List, Lit, Stmt, Struct, Ty},
    err::ParseError,
    token::{Tok, TokKind},
};
pub struct Parser {
    lx: crate::lex::Lex,
}

impl Parser {
    pub fn new(lx: crate::lex::Lex) -> Self {
        Self { lx }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        loop {
            stmts.push(match self.lx.look_ahead()? {
                tok if tok.is_str() => Stmt::Expr(self.parse_lit_str()?),
                Tok::Struct => Stmt::Decl(self.parse_struct()?),
                Tok::Let => Stmt::Decl(self.parse_let()?),
                Tok::EOF => return Ok(stmts),
                t => {
                    debug!("Parsing :: {:#?}", t);
                    return Err(ParseError::Unimplemented);
                }
            });
        }
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

        match self.lx.look_ahead()? {
            Tok::Semi | Tok::RBrace => {
                parse!("Found `;` or `}}`");
                return Ok(Struct {
                    name,
                    children,
                    files,
                });
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
                        });
                    }
                    Tok::FilePath(path) => {
                        parse!("Found file path :: {}", path);
                        files.push(path);
                    }
                    Tok::Ident(n) => children.push(self.rec_parse_struct(n)?),
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

    fn recursive_parse_struct(&mut self) -> Result<Struct, ParseError> {
        let name = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };

        let mut children = Vec::new();
        let mut files = Vec::new();

        let next = self.lx.next_token()?;
        match next {
            Tok::Semi => {
                return Ok(Struct {
                    name,
                    children,
                    files,
                })
            }
            Tok::LBrace => {}
            Tok::RBrace => {
                return Ok(Struct {
                    name,
                    children,
                    files,
                })
            }
            t => {
                parse!("Error Parsing :: {:#?}", t);
                return Err(ParseError::Expected {
                    line: self.lx.cx.line,
                    col: self.lx.cx.col,
                    expected: "struct or file ending".to_string(),
                    found: self.lx.look_ahead()?.to_string(),
                });
            }
        }

        loop {
            let path = match self.lx.next_token()? {
                Tok::RBrace => {
                    return Ok(Struct {
                        name,
                        children,
                        files,
                    })
                }
                Tok::Ident(name) => name,
                t => {
                    parse!("Error Parsing :: {:#?}", t);
                    return Err(ParseError::Expected {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        expected: "struct name or file ending".to_string(),
                        found: self.lx.look_ahead()?.to_string(),
                    });
                }
            };

            match self.lx.next_token()? {
                Tok::LBrace => {
                    let child = self.recursive_parse_struct()?;
                    children.push(child);
                }
                Tok::Dot => {
                    let _ = self.lx.assert_next_token(TokKind::Dot)?;
                    let ending = match self.lx.assert_next_token(TokKind::Ident)? {
                        Tok::Ident(ending) => ending,
                        _ => unreachable!(),
                    };
                    files.push(Box::new(format!("{}.{}", path, ending)).into_boxed_str());
                }
                Tok::RBrace => {
                    files.push(path);
                    return Ok(Struct {
                        name,
                        children,
                        files,
                    });
                }
                _ => {
                    return Err(ParseError::Expected {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        expected: "struct name or file ending".to_string(),
                        found: self.lx.look_ahead()?.to_string(),
                    })
                }
            }
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

        Ok(Decl::Let { name, ty, expr })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        parse!("Parsing expr");
        match self.lx.look_ahead()? {
            t if t.is_str() => self.parse_lit_str(),
            Tok::LBracket => Ok(Expr::Lit(Lit::List(self.parse_list()?))),
            Tok::Ident(_) => todo!(),
            t => {
                parse!("Parsing expr :: {:#?}", t);
                todo!()
            }
        }
    }

    fn parse_lit_str(&mut self) -> Result<Expr, ParseError> {
        parse!("Parsing lit str");
        let mut ix = 0;
        let mut out_body = Vec::new();
        let mut inserts = Vec::new();

        loop {
            match self.lx.next_token()? {
                Tok::String(body) => return Ok(Expr::Lit(Lit::Str(body))),
                Tok::DollarStarted(body) => {
                    out_body.push(body);
                    return Ok(Expr::Inserted {
                        string_parts: out_body,
                        inserts,
                    });
                }
                Tok::InBetween(body) | Tok::DollarTerminated(body) => {
                    ix += 1;
                    out_body.push(body);
                }
                _ => inserts.push((ix, self.parse_insertion()?)),
            }
        }
    }

    fn parse_insertion(&mut self) -> Result<Insert, ParseError> {
        let ident = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(ident) => ident,
            _ => unreachable!(),
        };

        match self.lx.look_ahead()? {
            Tok::LParen => {
                let _ = self.lx.assert_next_token(TokKind::LParen)?;
                let mut args = Vec::new();
                loop {
                    if let Tok::RParen = self.lx.look_ahead()? {
                        let _ = self.lx.assert_next_token(TokKind::RParen)?;
                        match self.lx.look_ahead()? {
                            Tok::For => {
                                let _ = self.lx.assert_next_token(TokKind::For)?;
                                let item = match self.lx.assert_next_token(TokKind::Ident)? {
                                    Tok::Ident(item) => item,
                                    _ => unreachable!(),
                                };
                                let _ = self.lx.assert_next_token(TokKind::In)?;
                                let list = self.parse_list()?;
                                return Ok(Insert::For {
                                    expr: Box::new(Insert::Fmt { name: ident, args }),
                                    item,
                                    iter: list,
                                });
                            }
                            _ => return Ok(Insert::Fmt { name: ident, args }),
                        }
                    }
                    args.push(self.parse_expr()?);
                }
            }
            _ => Ok(Insert::Ident(ident)),
        }
    }

    fn parse_list(&mut self) -> Result<List, ParseError> {
        let _ = self.lx.assert_next_token(TokKind::LBracket)?;
        let mut items = Vec::new();
        loop {
            if let Tok::RBracket = self.lx.look_ahead()? {
                let _ = self.lx.assert_next_token(TokKind::RBracket)?;
                return Ok(List::LitList(items));
            }
            items.push(match self.lx.next_token()? {
                Tok::String(item) => item,
                _ => {
                    return Err(ParseError::Expected {
                        line: self.lx.cx.line,
                        col: self.lx.cx.col,
                        expected: "string".to_string(),
                        found: self.lx.look_ahead()?.to_string(),
                    })
                }
            });
        }
    }

    fn parse_ty(&mut self) -> Result<Ty, ParseError> {
        match self.lx.next_token()? {
            Tok::Ident(ty) => match &(*ty) {
                "str" => Ok(Ty::Str),
                "char" => Ok(Ty::Char),
                "int" => Ok(Ty::Int),
                "list" => Ok(Ty::List),
                _ => Ok(Ty::Struct(ty.to_string())),
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
    use crate::ast::Stmt;
    use crate::lex::Lex;
    use crate::test;

    macro_rules! ps_test {
           ($( $name:ident, $inp:expr, $expected:expr; )*) => {
               $(
                   #[test]
                   fn $name() {
                       test!(PAR, "Testing `{}` = `{}`", stringify!($name), $inp);
                       let lex = Lex::new($inp);
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
        Ok(vec![Stmt::Decl(Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![Struct {
                name: "world".to_string().into_boxed_str(),
                children: vec![],
                files: vec![],
            }],
            files: vec![],
        }))]);
        test_struct_nested,
        "struct hello { world { nested } }",
        Ok(vec![Stmt::Decl(Decl::Struct(Struct {
            name: "hello".to_string().into_boxed_str(),
            children: vec![Struct {
                name: "world".to_string().into_boxed_str(),
                children: vec![Struct {
                    name: "nested".to_string().into_boxed_str(),
                    children: vec![],
                    files: vec![],
                }],
                files: vec![],
            }],
            files: vec![],
        }))]);
    );

    #[test]
    fn test_let() {
        let inp = "let a = \"hello\"";
        let lex = Lex::new(inp);
        let mut par = Parser::new(lex);
        let out = par.parse();
        let exp = Ok(vec![Stmt::Decl(Decl::Let {
            name: "a".to_string().into_boxed_str(),
            ty: None,
            expr: Expr::Lit(Lit::Str("hello".to_owned().into_boxed_str())),
        })]);
        assert_eq!(out, exp);
    }
}
