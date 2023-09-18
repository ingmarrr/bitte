use std::ops::Deref;

use crate::{
    ast::{Decl, Expr, Lit, LitStr, Stmt, Struct, Ty},
    err::ParseError,
    token::{Tok, TokKind},
};

macro_rules! yeet {
    ($cx:expr, $lhs:expr, $rhs:pat, $expected:expr) => {{
        let temp = $lhs;
        match temp {
            $rhs => {}
            t => {
                return Err(ParseError::Expected {
                    line: $cx.line,
                    col: $cx.col,
                    expected: $expected.to_string(),
                    found: t.to_string(),
                })
            }
        }
        temp
    }};
}

pub struct Parser {
    lx: crate::lex::Lex,
}

impl Parser {
    pub fn new(lx: crate::lex::Lex) -> Self {
        Self { lx }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        match self.lx.look_ahead()? {
            Tok::Struct => stmts.push(Stmt::Decl(self.parse_struct()?)),
            Tok::Let => {
                let l = self.parse_let()?;
                stmts.push(Stmt::Decl(l));
            }
        }

        Err(ParseError::UnexpectedEOF {
            line: self.lx.cx.line,
            col: self.lx.cx.col,
        })
    }

    fn parse_struct(&mut self) -> Result<Decl, ParseError> {
        let _ = self.lx.assert_next_token(TokKind::Struct)?;
        let s = self.recursive_parse_struct()?;

        Ok(Decl::Struct(s))
    }

    fn recursive_parse_struct(&mut self) -> Result<Struct, ParseError> {
        let struct_name = self.lx.assert_next_token(TokKind::Ident)?;
        let name = match struct_name {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };

        let mut children = Vec::new();
        let mut files = Vec::new();

        let _ = self.lx.assert_next_token(TokKind::LBrace)?;

        loop {
            if let Tok::RBrace = self.lx.look_ahead()? {
                let _ = self.lx.assert_next_token(TokKind::RBrace)?;
                return Ok(Struct {
                    name,
                    children,
                    files,
                });
            }
            let name = match self.lx.assert_next_token(TokKind::Ident)? {
                Tok::Ident(name) => name,
                _ => unreachable!(),
            };

            match self.lx.look_ahead()? {
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
                    files.push(Box::new(format!("{}.{}", name, ending)).into_boxed_str());
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
        let _ = self.lx.assert_next_token(TokKind::Let)?;
        let name = match self.lx.assert_next_token(TokKind::Ident)? {
            Tok::Ident(name) => name,
            _ => unreachable!(),
        };

        let _ = self.lx.assert_next_token(TokKind::Colon)?;
        let ty = self.parse_ty()?;

        let _ = self.lx.assert_next_token(TokKind::Eq)?;
        let expr = self.parse_expr()?;

        Ok(Decl::Let(name, ty, expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        match self.lx.look_ahead() {
            Tok::DQ => self.parse_lit_str(),
            Tok::LBracket => self.parse_lit_list(),
            Tok::Num(_) => self.parse_lit_num(),
            Tok::Ident(_) => todo!(),
            _ => todo!(),
        }
    }

    fn parse_ty(&mut self) -> Result<Ty, ParseError> {
        match self.lx.next_token()? {
            Tok::Ident(ty) => match &(*ty) {
                "str" => Ok(Ty::Str),
                "char" => Ok(Ty::Char),
                "int" => Ok(Ty::Int),
                "list" => Ok(Ty::List),
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

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::ast::{Param, Stmt};
//    use crate::lex::Lex;
//    use crate::test;
//
//    macro_rules! ps_test {
//        ($( $name:ident, $inp:expr, $expected:expr; )*) => {
//            $(
//                #[test]
//                fn $name() {
//                    test!(PAR, "Testing `{}` = `{}`", stringify!($name), $inp);
//                    let lex = Lex::new($inp);
//                    let mut parser = Parser::new(lex);
//                    let def = parser.parse();
//                    assert_eq!(def, $expected);
//                }
//            )*
//        };
//        ($name:ident, $inp:expr, $expected:expr) => {
//            #[test]
//            fn $name() {
//                test!(PAR, "Testing `{}` = `{}`", stringify!($name), $inp);
//                let lex = Lex::new($inp);
//                let mut parser = Parser::new(lex);
//                let def = parser.parse();
//                assert_eq!(def, $expected);
//            }
//        };
//    }
//}
