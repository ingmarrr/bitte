use crate::charset::{is_vert_ws, is_ws};
use crate::err::{LxErr, LxErrKind, Trace};
use crate::token::{Literal, Source, StringTy, TokKind, Token};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct Pos {
    pub ix: usize,
    pub line: usize,
    pub col: usize,
    pub ch: char,
}

pub struct Lexer<'a> {
    pub src: &'a [u8],
    pub pos: Pos,
    pub tmppos: Option<Pos>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a [u8]) -> Lexer<'a> {
        Lexer {
            src,
            pos: Pos {
                ix: 0,
                line: 0,
                col: 0,
                ch: '\0',
            },
            tmppos: None,
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
        if let None = self.tmppos {
            self.tmppos = Some(self.pos.to_owned());
        }
        let tok = self.lx_tok()?;
        println!("Lookahead: {}", tok);
        Ok(tok)
    }

    pub fn next_token(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        if let Some(cx) = self.tmppos.take() {
            self.pos = cx;
        }
        let tok: Token<'a> = self.lx_tok()?;
        println!("Nexttoken: {}", tok);
        Ok(tok)
    }

    fn lx_tok(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.skip_ws();
        Ok(match self.peek().or(Some(b'\0')).unwrap() {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lx_ident()?,
            b'0'..=b'9' => self.lx_num()?,
            b'"' => self.lx_str(false)?,
            b'$' => match self.peek_n(1) {
                Some(b'}') => self.lx_str(true)?,
                Some(ch) => {
                    self.take();
                    Token {
                        src: self.src(&self.src[self.pos.ix..self.pos.ix + 1]),
                        val: None,
                        kind: TokKind::from(ch),
                    }
                }
                None => return Err(self.err(LxErrKind::UnexpectedEOF, self.pos.ix)),
            },
            ch => {
                self.take();
                println!("tok: {}", ch as char);
                // if self.pos.ix >= self.src.len() {
                //     return Token {
                //         src: self.src(&self.src[self.pos.ix..]),
                //         val: None,
                //         kind: TokKind::from(ch),
                //     };
                // } else {

                // }
                println!("{}, {}", self.pos.ix, self.src.len());
                let src = if self.pos.ix >= self.src.len() {
                    self.src(&self.src[self.pos.ix..])
                } else {
                    self.src(&self.src[self.pos.ix..self.pos.ix + 1])
                };
                Token {
                    src,
                    val: None,
                    kind: TokKind::from(ch),
                }
            }
        })
    }

    fn lx_str(&mut self, insert_started: bool) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.take();
        if insert_started {
            self.take();
        }
        let six = self.pos.ix;

        while let Some(ch) = self.peek() {
            let token_kind = match ch {
                b'"' => Some(TokKind::Literal(Literal::String(match insert_started {
                    true => StringTy::InsertStarted,
                    false => StringTy::Literal,
                }))),
                b'{' => match self.peek_n(1) {
                    Some(b'$') => Some(TokKind::Literal(Literal::String(match insert_started {
                        true => StringTy::InBetween,
                        false => StringTy::InsertEnded,
                    }))),
                    _ => None,
                },
                _ => None,
            };
            let buf = &self.src[six..self.pos.ix];
            self.take();
            if let Some(kind) = token_kind {
                return Ok(Token {
                    src: self.src(&buf),
                    val: Some(self.to_str_or(buf)?),
                    kind,
                });
            }
        }

        Err(self.err(LxErrKind::UnterminatedString, six))
    }

    fn lx_ident(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        let six = self.pos.ix;
        while let Some(ch) = self.peek() {
            if !(b'a' <= ch && ch <= b'z' || b'A' <= ch && ch <= b'Z' || ch == b'_') {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.pos.ix];
        let val = self.to_str_or(buf)?;

        Ok(Token::<'a> {
            src: self.src(&buf),
            val: Some(val),
            kind: TokKind::from(val),
        })
    }

    fn lx_num(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        let six = self.pos.ix;
        while let Some(ch) = self.peek() {
            if !(b'0' <= ch && ch <= b'9') {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.pos.ix];
        Ok(Token {
            src: self.src(&buf),
            val: Some(std::str::from_utf8(buf).unwrap()),
            kind: TokKind::Literal(Literal::Int),
        })
    }

    fn take(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        if is_vert_ws(ch) {
            self.pos.line += 1;
            self.pos.col = 0;
        }
        self.pos.col += 1;
        self.pos.ix += 1;
        Some(ch)
    }

    fn take_or(&mut self, kind: LxErrKind) -> Result<u8, Trace<'a, LxErr>> {
        self.take().ok_or(self.err(kind, self.pos.ix))
    }

    fn take_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        let len = ch.len_utf8();
        self.pos.ix += len;
        Some(ch)
    }

    fn peek(&self) -> Option<u8> {
        if self.pos.ix >= self.src.len() {
            return None;
        }
        Some(self.src[self.pos.ix])
    }

    fn peek_char(&self) -> Option<char> {
        let rem = &self.src[self.pos.ix..];
        match std::str::from_utf8(&rem[..std::cmp::min(4, rem.len())]) {
            Ok(s) => s.chars().next(),
            Err(_) => None,
        }
    }

    fn peek_n(&self, n: usize) -> Option<u8> {
        if self.pos.ix + n >= self.src.len() {
            return None;
        }
        Some(self.src[self.pos.ix])
    }

    fn peek_or(&self, kind: LxErrKind) -> Result<u8, Trace<'a, LxErr>> {
        let res = self.peek();
        println!("{:#?}", res.unwrap_or(b'?') as char);
        res.ok_or(self.err(kind, self.pos.ix))
        // self.peek().ok_or(self.err(kind, self.pos.ix))
    }

    fn err(&self, kind: LxErrKind, six: usize) -> Trace<'a, LxErr> {
        let src = self.src(&self.src[six..self.pos.ix]);
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
            bix: self.pos.ix - buf.len(),
            col: self.pos.col,
            line: self.pos.line,
            src: &buf,
        }
    }

    fn to_str_or(&self, buf: &'a [u8]) -> Result<&'a str, Trace<'a, LxErr>> {
        std::str::from_utf8(buf).map_err(|_| self.err(LxErrKind::InvalidUtf8, self.pos.ix))
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
