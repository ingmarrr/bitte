use crate::{
    debug,
    err::LexError,
    info,
    token::{Tok, TokKind},
};

#[derive(Clone)]
pub struct Cx {
    pub ix: usize,
    pub line: usize,
    pub col: usize,
    pub ch: char,
}

pub struct Lexer {
    pub chars: Vec<char>,
    pub cx: Cx,
    pub tmpcx: Option<Cx>,
}

impl Lexer {
    pub fn new(inp: &str) -> Lexer {
        Lexer {
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

    pub fn look_ahead(&mut self) -> Result<Tok, LexError> {
        if let None = self.tmpcx {
            self.tmpcx = Some(self.cx.clone());
        }
        self.lx_tok()
    }

    pub fn assert_next_token(&mut self, expected: TokKind) -> Result<Tok, LexError> {
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

    pub fn next_token(&mut self) -> Result<Tok, LexError> {
        if let Some(cx) = &self.tmpcx {
            self.cx = cx.clone();
            self.tmpcx = None;
        }
        self.lx_tok()
    }

    fn lx_tok(&mut self) -> Result<Tok, LexError> {
        self.skip_ws();

        loop {
            let ch = match self.peek() {
                Some(c) => c,
                None => return Ok(Tok::EOF),
            };
            info!("LX_TOK :: {}", ch);
            match Tok::from(ch) {
                Tok::DQ | Tok::Dollar => return self.lx_str(),
                Tok::Char(c) if c.is_alphanumeric() => return self.lx_ident(),
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

        let dollar_started = match self.take() {
            Some('$') => true,
            Some('"') => false,
            _ => {
                debug!("HERE :: {}", buf);
                return Err(LexError::UnexpectedEOF {
                    line: self.cx.line,
                    col: self.cx.col,
                });
            }
        };

        loop {
            let ch = self.peek().ok_or(LexError::UnterminatedString {
                line: self.cx.line,
                col: self.cx.col,
            })?;
            match ch {
                '"' => {
                    self.take();
                    if dollar_started {
                        return Ok(Tok::DollarStarted(buf.into_boxed_str()));
                    }
                    return Ok(Tok::String(buf.into_boxed_str()));
                }
                '\\' => {
                    let next = self.peek();
                    match next {
                        Some('n') | Some('r') | Some('t') | Some('\\') | Some('"') | Some('$') => {
                            self.take();
                            buf.push(next.unwrap());
                        }
                        Some(c) => {
                            return Err(LexError::InvalidEscapeSequence {
                                line: self.cx.line,
                                col: self.cx.col,
                                ch: c,
                            })
                        }
                        None => {
                            return Err(LexError::UnterminatedString {
                                line: self.cx.line,
                                col: self.cx.col,
                            })
                        }
                    }
                }
                '$' => {
                    self.take();
                    if dollar_started {
                        return Ok(Tok::InBetween(buf.into_boxed_str()));
                    }
                    return Ok(Tok::DollarTerminated(buf.into_boxed_str()));
                }
                _ => {
                    self.take();
                    buf.push(ch);
                }
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
                Some(c) if c.is_alphanumeric() => {
                    buf.push(c);
                    self.take();
                }
                Some('_') => {
                    buf.push('_');
                    self.take();
                }
                Some('.') => {
                    buf.push('.');
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
                let mut lex = Lexer::new($inp);
                assert_eq!(lex.next_token(), $out);
            }
        };
    }

    lex_test!(
        lex_str,
        "\"hello world.\"",
        Ok(Tok::String("hello world.".to_owned().into_boxed_str()))
    );
    lex_test!(
        lex_str_unterminated,
        "\"hello",
        Err(LexError::UnterminatedString { line: 0, col: 6 })
    );
    lex_test!(
        lex_str_insert,
        "\"hello $world\"",
        Ok(Tok::DollarTerminated("hello ".to_owned().into_boxed_str()))
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
        let mut lex = Lexer::new("= + ? ! { } [ ] ( )");
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
