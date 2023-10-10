use crate::charset::{is_vert_ws, is_ws};
use crate::err::{LxErrKind, LxError};
use crate::tok::{Literal, Source, StringTy, Symbol, TokKind, Token};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct Cx {
    pub ix: usize,
    pub line: usize,
    pub col: usize,
    pub ch: char,
}

impl std::fmt::Display for Cx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Cx {{ ix: {}, line: {}, col: {}, ch: {} }}",
            self.ix, self.line, self.col, self.ch,
        ))
    }
}

pub struct Lexer {
    pub src: String,
    pub chars: Vec<char>,
    pub cx: Cx,
    pub tmpcx: Option<Cx>,
}

impl Lexer {
    pub fn new(inp: String) -> Lexer {
        let chars = inp.chars().collect();
        Lexer {
            src: inp,
            chars,
            cx: Cx {
                ix: 0,
                line: 0,
                col: 0,
                ch: '\0',
            },
            tmpcx: None,
        }
    }

    pub fn look_ahead(&mut self) -> Result<Token, LxError> {
        if let None = self.tmpcx {
            self.tmpcx = Some(self.cx.to_owned());
        }
        let tok = self.lx_tok()?;
        Ok(tok)
    }

    pub fn next_token(&mut self) -> Result<Token, LxError> {
        if let Some(cx) = self.tmpcx.take() {
            self.cx = cx;
        }
        let tok = self.lx_tok()?;
        Ok(tok)
    }

    fn lx_tok(&mut self) -> Result<Token, LxError> {
        self.skip_ws();
        let ch = self.peek_or(LxErrKind::UnexpectedEOF)?;
        Ok(match ch {
            'a'..='z' | 'A'..='Z' | '_' => self.lx_ident()?,
            '0'..='9' => self.lx_num()?,
            '"' => self.lx_str(false)?,
            '$' => match self.peek_n(1) {
                Some('}') => self.lx_str(true)?,
                _ => Token {
                    src: self.src(&self.src[self.cx.ix..self.cx.ix + 1]),
                    val: None,
                    kind: TokKind::from(self.take_or(LxErrKind::UnexpectedEOF)?),
                },
            },
            _ => Token {
                src: self.src(&self.src[self.cx.ix..self.cx.ix + 1]),
                val: None,
                kind: TokKind::from(self.take_or(LxErrKind::UnexpectedEOF)?),
            },
        })
    }

    fn lx_str<'a>(&'a mut self, insert_started: bool) -> Result<Token<'a>, LxError> {
        let six = self.cx.ix;
        self.take();
        if insert_started {
            self.take();
        }

        while let Some(ch) = self.peek() {
            let token_kind = match ch {
                '"' => Some(TokKind::Literal(Literal::String(match insert_started {
                    true => StringTy::InsertStarted,
                    false => StringTy::Literal,
                }))),
                '{' => match self.peek_n(1) {
                    Some('$') => Some(TokKind::Literal(Literal::String(match insert_started {
                        true => StringTy::InBetween,
                        false => StringTy::InsertEnded,
                    }))),
                    _ => None,
                },
                _ => None,
            };
            self.take();
            if let Some(kind) = token_kind {
                let buf = &self.src[six..self.cx.ix];
                return Ok(Token {
                    src: self.src(&buf),
                    val: Some(buf.as_ref()),
                    kind,
                });
            }
        }

        Err(self.err(LxErrKind::UnterminatedString))
    }

    fn lx_ident(&mut self) -> Result<Token, LxError> {
        let six = self.cx.ix;
        while let Some(ch) = self.peek() {
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.cx.ix];
        Ok(Token {
            src: self.src(&buf),
            val: Some(&buf),
            kind: TokKind::Ident,
        })
    }

    fn lx_num(&mut self) -> Result<Token, LxError> {
        let six = self.cx.ix;
        while let Some(ch) = self.peek() {
            if !ch.is_digit(10) {
                break;
            }
            self.take();
        }

        let buf = &self.src[six..self.cx.ix];
        Ok(Token {
            src: self.src(&buf),
            val: Some(&buf),
            kind: TokKind::Literal(Literal::Int),
        })
    }

    fn take(&mut self) -> Option<char> {
        let ch = self.peek()?;
        if is_vert_ws(ch) {
            self.cx.line += 1;
            self.cx.col = 0;
        } else {
            self.cx.col += 1;
        }
        lex!(
            "[take] {} :: [line] {} :: [col] {} ",
            ch,
            self.cx.line,
            self.cx.col
        );

        Some(ch)
    }

    fn take_or(&mut self, kind: LxErrKind) -> Result<char, LxError> {
        self.take().ok_or(self.err(kind))
    }

    fn peek(&mut self) -> Option<char> {
        if self.cx.ix >= self.chars.len() {
            return None;
        }
        Some(self.chars[self.cx.ix])
    }

    fn peek_n(&mut self, n: usize) -> Option<char> {
        if self.cx.ix + n >= self.chars.len() {
            return None;
        }
        Some(self.chars[self.cx.ix])
    }

    fn peek_or(&mut self, kind: LxErrKind) -> Result<char, LxError> {
        self.peek().ok_or(self.err(kind))
    }

    fn err(&self, kind: LxErrKind) -> LxError {
        match kind {
            LxErrKind::InvalidToken => LxError::InvalidToken(self.cx.clone()),
            LxErrKind::InvalidCharacter => LxError::InvalidCharacter(self.cx.clone()),
            LxErrKind::UnexpectedEOF => LxError::UnexpectedEOF(self.cx.clone()),
            LxErrKind::UnterminatedString => LxError::Unterminated(self.cx.clone()),
        }
    }

    fn src(&self, buf: &str) -> Source {
        Source {
            bix: self.cx.ix - buf.len(),
            eix: self.cx.ix,
        }
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
