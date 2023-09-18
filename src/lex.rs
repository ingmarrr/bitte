use crate::{
    err::LexError,
    error, info, log,
    token::{InsertKw, Tok, TokKind},
};

#[derive(Clone)]
pub struct Cx {
    pub ix: usize,
    pub line: usize,
    pub col: usize,
    pub ch: char,
}

pub struct Lex {
    pub chars: Vec<char>,
    pub cx: Cx,
    pub tmpcx: Option<Cx>,
}

impl Lex {
    pub fn new(inp: &str) -> Lex {
        Lex {
            chars: inp.chars().collect(),
            cx: Cx {
                ix: 0,
                line: 0,
                col: 0,
                ch: '\0',
            },
            tmpcx: None,
        }
    }

    pub fn assert_look_ahead(&mut self, expected: Tok) -> Result<Tok, LexError> {
        if let Some(cx) = &self.tmpcx {
            self.cx = cx.clone();
            self.tmpcx = None;
        }
        let tok = self.lx_tok()?;
        if tok != expected {
            return Err(LexError::Expected {
                line: self.cx.line,
                col: self.cx.col,
                expected: expected.to_string(),
                found: tok.to_string(),
            });
        }
        Ok(tok)
    }

    pub fn look_ahead(&mut self) -> Result<Tok, LexError> {
        if let Some(cx) = &self.tmpcx {
            self.cx = cx.clone();
            self.tmpcx = None;
        }
        self.lx_tok()
    }

    pub fn assert_next_token(&mut self, expected: TokKind) -> Result<Tok, LexError> {
        if let None = self.tmpcx {
            self.tmpcx = Some(self.cx.clone());
        }
        let tok = self.lx_tok()?;
        if tok != expected {
            return Err(LexError::Expected {
                line: self.cx.line,
                col: self.cx.col,
                expected: expected.to_string(),
                found: tok.to_string(),
            });
        }
        Ok(tok)
    }

    pub fn next_token(&mut self) -> Result<Tok, LexError> {
        if let None = self.tmpcx {
            self.tmpcx = Some(self.cx.clone());
        }
        self.lx_tok()
    }

    fn lx_tok(&mut self) -> Result<Tok, LexError> {
        self.skip_ws();

        loop {
            let ch = self.peek().ok_or(LexError::EOF)?;
            info!("LX_TOK :: {}", ch);
            match Tok::from(ch) {
                Tok::DQ => return self.lx_str(),
                Tok::Char(_) => return self.lx_ident(),
                t => {
                    self.take();
                    return Ok(t);
                }
            }
        }
    }

    fn lx_str(&mut self) -> Result<Tok, LexError> {
        info!("LX_STR");
        let mut buf = String::new();
        let mut chs = String::new();
        let mut inserts = Vec::new();

        self.take().ok_or(LexError::UnexpectedEOF {
            line: self.cx.line,
            col: self.cx.col,
        })?;

        loop {
            let ch = self.peek().ok_or(LexError::UnterminatedString {
                line: self.cx.line,
                col: self.cx.col,
            })?;
            match ch {
                '"' => {
                    self.take();
                    return Ok(Tok::String { body: buf, inserts });
                }
                '$' => inserts.push(self.lx_ins()?),
                _ => {
                    self.take();
                    buf.push(ch);
                    chs.push(ch);
                }
            }
        }
    }

    fn lx_ins(&mut self) -> Result<Tok, LexError> {
        info!("LX_INS");

        self.take().ok_or(LexError::UnexpectedEOF {
            line: self.cx.line,
            col: self.cx.col,
        })?;
        let start = self.cx.ix;

        match self.peek() {
            Some('$') => {
                info!("LX_INS :: $$");
                self.take();
                return Ok(Tok::Insert {
                    kw: InsertKw::None,
                    start,
                });
            }
            Some(c) if c.is_alphabetic() => match self.lx_ident()? {
                Tok::For => {
                    let list = self.lx_ident()?;
                    let name = match list {
                        Tok::Ident(n) => n,
                        _ => {
                            return Err(LexError::Expected {
                                line: self.cx.line,
                                col: self.cx.col,
                                expected: "identifier".to_string(),
                                found: format!("{:?}", list),
                            })
                        }
                    };
                    info!("LX_INS :: $for {}", name);
                    let fmt = match self.lx_str()? {
                        Tok::String { body, inserts } => Tok::String { body, inserts },
                        _ => unreachable!(),
                    };
                    info!("LX_INS :: fmt {:?}", fmt);
                    match self.peek() {
                        Some('$') => {
                            self.take();
                        }
                        _ => {
                            error!("LX_INS :: expected $");
                            return Err(LexError::UnterminatedInsertion {
                                line: self.cx.line,
                                col: self.cx.col,
                            });
                        }
                    }

                    return Ok(Tok::Insert {
                        kw: InsertKw::For {
                            name,
                            fmt: Box::new(fmt),
                        },
                        start,
                    });
                }
                Tok::Ident(i) => {
                    return Ok(Tok::Insert {
                        kw: InsertKw::Some(i),
                        start,
                    })
                }
                _ => {
                    return Err(LexError::InvalidInsertKeyword {
                        line: self.cx.line,
                        col: self.cx.col,
                    })
                }
            },
            _ => {
                return Err(LexError::UnterminatedInsertion {
                    line: self.cx.line,
                    col: self.cx.col,
                })
            }
        }
    }

    fn lx_ident(&mut self) -> Result<Tok, LexError> {
        info!("LX_IDENT");
        let mut buf = String::new();

        loop {
            let ch = self.peek();
            match ch {
                Some(c) if c.is_whitespace() => {
                    self.take();
                    info!("LX_IDENT :: {}", buf);
                    return Ok(Tok::from(buf));
                }
                Some(c) if c.is_alphabetic() => {
                    buf.push(c);
                    self.take();
                }
                Some('_') => {
                    buf.push('_');
                    self.take();
                }
                _ => {
                    info!("LX_IDENT :: {}", buf);
                    return Ok(Tok::from(buf));
                }
            }
        }
    }

    fn take(&mut self) -> Option<char> {
        let ch = self.peek()?;
        match ch {
            '\n' | '\r' => {
                self.cx.line += 1;
                self.cx.col = 0;
            }
            _ => {
                self.cx.col += 1;
            }
        }
        self.cx.ix += 1;
        info!(
            "TAKE :: {} :: LINE {} :: COL {}",
            ch, self.cx.line, self.cx.col
        );
        Some(ch)
    }

    fn peek(&mut self) -> Option<char> {
        if self.cx.ix >= self.chars.len() {
            return None;
        }
        Some(self.chars[self.cx.ix])
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }
            self.take();
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test;

    macro_rules! lex_test {
        ($name:ident, $inp:expr, $out:expr) => {
            #[test]
            fn $name() {
                test!(LEX, "Testing `{}` = `{}`", stringify!($name), $inp);
                let mut lex = Lex::new($inp);
                assert_eq!(lex.next_token(), $out);
            }
        };
    }

    lex_test!(
        lex_str,
        "\"hello world.\"",
        Ok(Tok::String {
            body: "hello world.".to_string(),
            inserts: vec![]
        })
    );
    lex_test!(
        lex_str_unterminated,
        "\"hello",
        Err(LexError::UnterminatedString { line: 0, col: 6 })
    );
    lex_test!(
        lex_str_insert,
        "\"hello $world\"",
        Ok(Tok::String {
            body: "hello ".to_string(),
            inserts: vec![Tok::Insert {
                kw: InsertKw::Some("world".to_owned().into_boxed_str()),
                start: 8
            }]
        })
    );
    lex_test!(
        lex_str_insert_empty,
        r#""hello $$""#,
        Ok(Tok::String {
            body: "hello ".to_string(),
            inserts: vec![Tok::Insert {
                kw: InsertKw::None,
                start: 8
            },]
        })
    );
    lex_test!(
        lex_str_insert_num,
        r#""hello $1""#,
        Ok(Tok::String {
            body: "hello ".to_string(),
            inserts: vec![Tok::Insert {
                kw: InsertKw::Number(1),
                start: 8
            },]
        })
    );
    lex_test!(
        lex_str_insert_for,
        r#""hello $for world "there, $$"$""#,
        Ok(Tok::String {
            body: "hello ".to_string(),
            inserts: vec![Tok::Insert {
                kw: InsertKw::For {
                    name: "world".to_owned().into_boxed_str(),
                    fmt: Box::new(Tok::String {
                        body: "there, ".to_string(),
                        inserts: vec![Tok::Insert {
                            kw: InsertKw::None,
                            start: 27
                        },]
                    })
                },
                start: 8
            },]
        })
    );
    lex_test!(lex_struct, "struct", Ok(Tok::Struct));
    lex_test!(lex_fun, "fmt", Ok(Tok::Fmt));
    lex_test!(lex_let, "let", Ok(Tok::Let));
    lex_test!(
        lex_ident,
        "hello",
        Ok(Tok::Ident("hello".to_owned().into_boxed_str()))
    );

    #[test]
    fn lex_symbols() {
        let mut lex = Lex::new("= + ? ! { } [ ] ( )");
        assert_eq!(lex.next_token(), Ok(Tok::Eq));
        assert_eq!(lex.next_token(), Ok(Tok::Plus));
        assert_eq!(lex.next_token(), Ok(Tok::Opt));
        assert_eq!(lex.next_token(), Ok(Tok::Req));
        assert_eq!(lex.next_token(), Ok(Tok::LBrace));
        assert_eq!(lex.next_token(), Ok(Tok::RBrace));
        assert_eq!(lex.next_token(), Ok(Tok::LBracket));
        assert_eq!(lex.next_token(), Ok(Tok::RBracket));
        assert_eq!(lex.next_token(), Ok(Tok::LParen));
        assert_eq!(lex.next_token(), Ok(Tok::RParen));
    }
}
