use crate::charset::{is_vert_ws, is_ws};
use crate::err::{LxErr, LxErrKind, Trace};
use crate::fifo::Fifo;
use crate::token::{Source, TokKind, Token};

#[derive(Clone)]
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
            if let TokKind::EOF = tok.kind {
                break;
            }
            toks.push(tok);
        }

        Ok(toks)
    }

    pub fn look_ahead(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.tmpcx = Some(self.cx.clone());
        if self.cx.pending.has_some() {
            tilog::debug!(
                lex,
                "Lookahead (pending): {:#?}",
                self.cx.pending.peek().unwrap().to_string()
            );
            return Ok(self.cx.pending.pop_sure());
        }
        let tok = self.lx_tok()?;
        tilog::debug!(lex, "Lookahead: {}", tok);
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

    #[rustfmt::skip]
    pub fn try_lx_ident(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        self.reset();
        if let Ok(Token { kind: TokKind::Ident, .. }) = self.cx.pending.peek() {
            return Ok(self.cx.pending.pop_sure());
        }
        self.lx_ident()
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
            b'!' => match self.peek_n(1) {
                Some(b'=') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        kind: TokKind::Neq,
                    }
                }
                _ => {
                    self.take();
                    Token {
                        src: self.src_single(),
                        kind: TokKind::Bang,
                    }
                }
            },
            b'=' => match self.peek_n(1) {
                Some(b'=') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        kind: TokKind::Eq,
                    }
                }
                _ => {
                    self.take();
                    Token {
                        src: self.src_single(),
                        kind: TokKind::Eq,
                    }
                }
            },
            b'{' => match self.peek_n(1) {
                Some(b'{') => {
                    self.take();
                    self.take();

                    let src = self.src_double();
                    let st = self.lx_str(false)?;
                    self.cx.pending.push(st);
                    Token {
                        src,
                        kind: TokKind::LCurlyDouble,
                    }
                }
                Some(b'$') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        kind: TokKind::LCurlyDollar,
                    }
                }
                _ => {
                    self.take();
                    Token {
                        src: self.src_single(),
                        kind: TokKind::LCurly,
                    }
                }
            },
            b'}' => match self.peek_n(1) {
                Some(b'}') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        kind: TokKind::RCurlyDouble,
                    }
                }
                _ => {
                    self.take();
                    Token {
                        src: self.src_single(),
                        kind: TokKind::RCurly,
                    }
                }
            }
            b'"' => {
                self.take();
                let src = self.src_single();

                let string = self.lx_str(true)?;
                self.cx.pending.push(string);

                self.take();
                self.cx.pending.push(Token {
                    src: self.src_single(),
                    kind: TokKind::CloserDQuote,
                });

                Token {
                    src,
                    kind: TokKind::OpenerDQuote,
                }
            },
            b'$' => match self.peek_n(1) {
                Some(b'}') => {
                    self.take();
                    self.take();
                    Token {
                        src: self.src_double(),
                        kind: TokKind::RCurlyDollar,
                    }
                }
                Some(ch) => {
                    self.take();
                    Token {
                        src: self.src_single(),
                        kind: TokKind::from(ch),
                    }
                }
                None => return Err(self.err(LxErrKind::UnexpectedEOF, self.cx.ix)),
            },
            ch => {
                self.take();
                Token {
                    src: if self.src.len() == 0 {
                        self.src(&self.src[self.cx.ix..])
                    } else {
                        self.src_single()
                    },
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
                b'}' => match self.peek_n(1) {
                    Some(b'}') => Some(TokKind::StringLit),
                    _ => None,
                },
                b'{' => match self.peek_n(1) {
                    Some(b'$') => Some(TokKind::StringLit),
                    _ => None,
                },
                b'@' => {
                    self.take();
                    let next = self.lx_ident()?;
                    tilog::info!(lex, "Next: `{}`", next.val());
                    match next.kind {
                        TokKind::For 
                        | TokKind::If 
                        | TokKind::Else 
                        | TokKind::ElseIf => Some(next.kind),
                        _ => None
                    }
                }
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


    #[rustfmt::skip]
    fn lx_ident(&mut self) -> Result<Token<'a>, Trace<'a, LxErr>> {
        let six = self.cx.ix;
        let is_num = |ch| b'0' <= ch && ch <= b'9' 
            || ch == b'.';
        let is_ident = |ch| b'a' <= ch && ch <= b'z' 
            || b'A' <= ch && ch <= b'Z' 
            || ch == b'_' 
            || ch == b'.';

        if let Some(ch) = self.peek() {
            if is_ident(ch) {
                self.take();
            }
        }

        while let Some(ch) = self.peek() {
            if !(is_ident(ch) || is_num(ch)) {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.cx.ix];
        let val = self.try_to_str(buf)?;

        Ok(Token::<'a> {
            src: self.src(&buf),
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
            kind: TokKind::IntLit,
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

    fn peek(&self) -> Option<u8> {
        if self.cx.ix >= self.src.len() {
            return None;
        }
        Some(self.src[self.cx.ix])
    }

    fn peek_n(&self, n: usize) -> Option<u8> {
        if self.cx.ix + n >= self.src.len() {
            return None;
        }
        Some(self.src[self.cx.ix + n])
    }

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

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_symbols() {
        let src = "!@#$()[]{},.;:=";
        let mut lx = Lexer::new(src.as_bytes());
        let toks = lx.lex().unwrap();
        assert_eq!(toks.len(), src.len());
        for (i, tok) in toks.iter().enumerate() {
            assert_eq!(tok.src.buf[0], src.as_bytes()[i]);
        }
    }

    #[test]
    fn test_idents() {
        let src =
            "abc def ghi jkl mno pqr stu vwx yz ABC DEF GHI JKL MNO PQR STU VWX YZ _ _a _0 _a0";
        let inp = src.split(' ').into_iter().collect::<Vec<&str>>();
        let mut lx = Lexer::new(src.as_bytes());
        let toks = lx.lex().unwrap();
        println!("{:#?}", toks);
        assert_eq!(toks.len(), inp.len());
        for (i, tok) in toks.iter().enumerate() {
            assert!(tok.val() == inp[i], "[{}]", tok);
        }
    }

    #[test]
    fn test_keywords() {
        let src = "main let dir file req for in if else elseif str list";
        let inp = src.split(' ').into_iter().collect::<Vec<&str>>();
        let mut lx = Lexer::new(src.as_bytes());
        let toks = lx.lex().unwrap();
        println!("{:#?}", toks);
        assert_eq!(toks.len(), inp.len());
        for (i, tok) in toks.iter().enumerate() {
            assert!(tok.val() == inp[i], "[{}]", tok);
        }
    }
    
}
