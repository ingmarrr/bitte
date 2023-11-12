use crate::charset::{is_vert_ws, is_ws};
use crate::err::{LxErr, LxErrKind, Trace};
use crate::fifo::Fifo;
use crate::token::{Source, TokKind, Token};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct Cx<'a> {
    pub ix: usize,
    pub line: usize,
    pub col: usize,
    pub ch: char,
    pub pending: Fifo<Token<'a>>,
}

pub struct Lexer<'a> {
    pub src: &'a [u8],
    pub cx: Cx<'a>,
    pub tmpcx: Option<Cx<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a [u8]) -> Lexer<'a> {
        Lexer {
            src,
            cx: Cx {
                ix: 0,
                line: 0,
                col: 0,
                ch: '\0',
                pending: Fifo::new(),
            },
            tmpcx: None,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token<'a>>, Trace<'a, LxErr>> {
        let mut toks = Vec::new();

        while let Ok(tok) = self.next_token() {
            toks.push(tok);
        }

        Ok(toks)
    }

    pub fn look_ahead(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.tmpcx = Some(self.cx.to_owned());
        if self.cx.pending.has_some() {
            return Ok(self.cx.pending.pop_sure());
        }
        let tok = self.lx_tok()?;
        self.reset();
        Ok(tok)
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.reset();
        let tok: Token<'a> = if self.cx.pending.has_some() {
            tilog::debug!(
                lex,
                "Nexttoken (pending): {:#?}",
                self.cx.pending.peek().unwrap().to_string()
            );
            self.cx.pending.pop_sure()
        } else {
            self.lx_tok()?
        };
        tilog::debug!(lex, "Nexttoken: {}", tok);
        Ok(tok)
    }

    #[rustfmt::skip]
    pub fn try_lx_str(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.reset();
        if let Ok(Token { kind: TokKind::StringLit, .. }) = self.cx.pending.peek() {
            return Ok(self.cx.pending.pop_sure());
        }
        self.lx_str(false)
    }

    pub fn reset(&mut self) {
        if let Some(cx) = self.tmpcx.take() {
            self.cx = cx;
        }
    }

    fn lx_tok(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.skip_ws();
        Ok(match self.peek().or(Some(b'\0')).unwrap() {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lx_ident()?,
            b'0'..=b'9' => self.lx_num()?,
            tok if tok == b'{' => match self.peek_n(1) {
                Some(b'"') => {
                    self.take();
                    self.take();

                    let tok = Token {
                        src: self.src_double(),
                        kind: TokKind::LCurlyDQuote,
                    };
                    let st = self.lx_str(false)?;
                    let _ = self.cx.pending.push(st);
                    tok
                }
                Some(b'$') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        // val: None,
                        kind: TokKind::LCurlyDollar,
                    }
                }
                _ => {
                    let _ = self.take();
                    Token {
                        src: self.src_single(),
                        kind: TokKind::LCurly,
                    }
                }
            },
            b'"' => match self.peek_n(1) {
                Some(b'}') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        kind: TokKind::RCurlyDQuote,
                    }
                }
                _ => {
                    self.take();
                    let tok = Token {
                        src: self.src_single(),
                        // val: None,
                        kind: TokKind::OpenerDQuote,
                    };
                    let st = self.lx_str(true)?;
                    self.cx.pending.push(st);
                    self.take();
                    self.cx.pending.push(Token {
                        src: self.src_single(),
                        // val: None,
                        kind: TokKind::CloserDQuote,
                    });
                    tok
                }
            },
            b'$' => match self.peek_n(1) {
                Some(b'}') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        // val: None,
                        kind: TokKind::RCurlyDollar,
                    }
                }
                Some(ch) => {
                    self.take();
                    Token {
                        src: self.src_single(),
                        // val: None,
                        kind: TokKind::from(ch),
                    }
                }
                None => return Err(self.err(LxErrKind::UnexpectedEOF, self.cx.ix)),
            },
            ch => {
                // println!(
                //     "Tok: {}, Position: {}, Length: {}",
                //     ch as char,
                //     self.cx.ix,
                //     self.src.len()
                // );
                self.take();
                Token {
                    src: if self.src.len() == 0 {
                        self.src(&self.src[self.cx.ix..])
                    } else {
                        self.src_single()
                    },
                    // val: None,
                    kind: TokKind::from(ch),
                }
            }
        })
    }

    fn lx_str(&mut self, raw_str: bool) -> Result<Token<'a>, Trace<'a, LxErr>> {
        let six = self.cx.ix;

        while let Some(ch) = self.peek() {
            let token_kind = match ch {
                b'"' if raw_str => Some(TokKind::StringLit),
                b'"' => match self.peek_n(1) {
                    Some(b'}') => Some(TokKind::StringLit),
                    _ => None,
                },
                b'{' => match self.peek_n(1) {
                    Some(b'$') => Some(TokKind::StringLit),
                    _ => None,
                },
                _ => None,
            };
            let buf = &self.src[six..self.cx.ix];
            if let Some(kind) = token_kind {
                return Ok(Token {
                    src: self.src(&buf),
                    kind,
                });
            }
            self.take();
        }

        Err(self.err(LxErrKind::UnterminatedString, six))
    }

    fn lx_ident(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        let six = self.cx.ix;
        while let Some(ch) = self.peek() {
            if !(b'a' <= ch && ch <= b'z' || b'A' <= ch && ch <= b'Z' || ch == b'_') {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.cx.ix];
        let val = self.try_to_str(buf)?;

        Ok(Token::<'a> {
            src: self.src(&buf),
            // val: Some(val),
            kind: TokKind::from(val),
        })
    }

    fn lx_num(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        let six = self.cx.ix;
        while let Some(ch) = self.peek() {
            if !(b'.' <= ch && ch <= b'9') {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.cx.ix];
        Ok(Token {
            src: self.src(&buf),
            // val: Some(std::str::from_utf8(buf).unwrap()),
            kind: TokKind::Int,
        })
    }

    fn take(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        if is_vert_ws(ch) {
            self.cx.line += 1;
            self.cx.col = 0;
        }
        self.cx.col += 1;
        self.cx.ix += 1;
        Some(ch)
    }

    // fn take_or(&mut self, kind: LxErrKind) -> Result<u8, Trace<'a, LxErr>> {
    //     self.take().ok_or(self.err(kind, self.pos.ix))
    // }

    // fn take_char(&mut self) -> Option<char> {
    //     let ch = self.peek_char()?;
    //     let len = ch.len_utf8();
    //     self.pos.ix += len;
    //     Some(ch)
    // }

    fn peek(&self) -> Option<u8> {
        if self.cx.ix >= self.src.len() {
            return None;
        }
        Some(self.src[self.cx.ix])
    }

    // fn peek_char(&self) -> Option<char> {
    //     let rem = &self.src[self.pos.ix..];
    //     match std::str::from_utf8(&rem[..std::cmp::min(4, rem.len())]) {
    //         Ok(s) => s.chars().next(),
    //         Err(_) => None,
    //     }
    // }

    fn peek_n(&self, n: usize) -> Option<u8> {
        if self.cx.ix + n >= self.src.len() {
            return None;
        }
        Some(self.src[self.cx.ix + n])
    }

    // fn peek_or(&self, kind: LxErrKind) -> Result<u8, Trace<'a, LxErr>> {
    //     let res = self.peek();
    //     println!("{:#?}", res.unwrap_or(b'?') as char);
    //     res.ok_or(self.err(kind, self.pos.ix))
    //     self.peek().ok_or(self.err(kind, self.pos.ix))
    // }

    fn err(&self, kind: LxErrKind, six: usize) -> Trace<'a, LxErr> {
        let src = self.src(&self.src[six..self.cx.ix]);
        let src_str = src.to_string();
        Trace {
            src,
            err: match kind {
                LxErrKind::InvalidToken => LxErr::InvalidToken(src_str),
                LxErrKind::InvalidCharacter => LxErr::InvalidCharacter(src_str),
                LxErrKind::InvalidUtf8 => LxErr::InvalidUtf8(src_str),
                LxErrKind::UnexpectedEOF => LxErr::UnexpectedEOF(src_str),
                LxErrKind::UnterminatedString => LxErr::Unterminated(src_str),
            },
        }
    }

    fn src(&self, buf: &'a [u8]) -> Source<'a> {
        Source {
            bix: self.cx.ix - buf.len(),
            col: self.cx.col - buf.len(),
            line: self.cx.line,
            buf: &buf,
            len: buf.len(),
        }
    }

    fn src_double(&self) -> Source<'a> {
        Source {
            bix: self.cx.ix - 2,
            col: self.cx.col - 2,
            line: self.cx.line,
            buf: &self.src[self.cx.ix - 2..self.cx.ix],
            len: 2,
        }
    }

    fn src_single(&self) -> Source<'a> {
        Source {
            bix: self.cx.ix - 1,
            col: self.cx.col - 1,
            line: self.cx.line,
            buf: &self.src[self.cx.ix - 1..self.cx.ix],
            len: 1,
        }
    }

    fn try_to_str(&self, buf: &'a [u8]) -> Result<&'a str, Trace<'a, LxErr>> {
        std::str::from_utf8(buf).map_err(|_| self.err(LxErrKind::InvalidUtf8, self.cx.ix))
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if !is_ws(ch) {
                break;
            }
            self.take();
        }
    }
}
