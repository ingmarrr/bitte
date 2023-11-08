use std::error::Error;

use crate::{
    consts,
    token::{Source, Token},
};

pub enum LxErrKind {
    InvalidToken,
    InvalidCharacter,
    InvalidUtf8,
    UnexpectedEOF,
    UnterminatedString,
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum LxErr {
    #[error("Lex: Invalid token :: {0}")]
    InvalidToken(String),
    #[error("Lex: Invalid character :: {0}")]
    InvalidCharacter(String),
    #[error("Lex: Invalid Utf8 :: {0}")]
    InvalidUtf8(String),
    #[error("Lex: Unexpected EOF :: {0}")]
    UnexpectedEOF(String),
    #[error("Lex: Unterminated String :: {0}")]
    Unterminated(String),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, thiserror::Error)]
pub enum SynErr {
    #[error("Syntax Error: Expected [{0}], found [{1}] :: {2}")]
    Expected(String, String, String),

    #[error(transparent)]
    LxErr(#[from] LxErr),
}

// #[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, thiserror::Error)]
pub enum ExecErr {
    #[error("Exec: Expected [{0}], found [{1}] :: {2}")]
    Expected(String, String, String),

    #[error("Exec: AlreadyExists :: {0}")]
    AlreadyExists(String),

    #[error("Program cannot contain multiple main declarations")]
    MultipleMain,

    #[error("Exec: NotFound :: {0}")]
    NotFound(String),

    #[error("Exec: InvalidType :: {0}, expected :: {1}")]
    InvalidType(String, String),

    #[error("Exec: InvalidArgument :: {0}")]
    InvalidArgument(String),

    #[error(transparent)]
    LxErr(#[from] LxErr),

    #[error(transparent)]
    SynErr(#[from] SynErr),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DbErr {
    #[error("Home Variable Not Found")]
    HomeNotFound,

    #[error("Db: NotFound :: {0}")]
    NotFound(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug)]
pub struct Trace<'a, E>
where
    E: Error + 'static,
{
    pub src: Source<'a>,
    pub err: E,
}

impl<'a, E> Trace<'a, E>
where
    E: Error + 'static,
{
    pub fn new(src: Source<'a>, err: E) -> Self {
        Self { src, err }
    }

    pub fn display_line(&self, src: &'a [u8]) -> String {
        let mut line = String::new();
        let mut start_ix = self.src.bix;
        let mut end_ix = self.src.bix + self.src.len - 1;

        loop {
            if src[start_ix] == b'\n' || start_ix == 0 {
                break;
            }
            start_ix -= 1;
        }

        loop {
            if src[end_ix] == b'\n' || end_ix == src.len() - 1 {
                break;
            }
            end_ix += 1;
        }

        for i in start_ix..=end_ix {
            if i >= self.src.bix && i < self.src.bix + self.src.len {
                line.push_str(consts::RED);
            } else {
                line.push_str(consts::GREEN);
            }
            line.push(src[i] as char);
            if i == self.src.bix {
                line.push_str(consts::RESET);
            }
        }
        line.push_str(consts::RESET);

        line.push('\n');

        for i in 0..line.len() {
            if i < self.src.bix - start_ix {
                line.push(' ');
            } else if i >= self.src.bix - start_ix && i < self.src.bix - start_ix + self.src.len {
                line.push_str(consts::RED);
                line.push('^');
                line.push_str(consts::RESET);
            } else {
                break;
            }
        }

        line
    }

    pub fn dump_err(&self, src: &'a [u8]) -> String {
        let mut err = String::new();
        err.push_str(&self.display_line(src));
        err.push_str("\n");
        err.push_str(&self.err.to_string());
        err
    }

    pub fn dump(&self, src: &'a [u8]) {
        println!("{}", self.display_line(src));
        println!("{}", self.err);
    }
}

impl<'a> Trace<'a, SynErr> {
    pub fn new_syn(tok: Token<'a>, msg: &str) -> Trace<'a, SynErr> {
        Self {
            src: tok.src,
            err: SynErr::Expected(msg.to_string(), tok.kind.to_string(), tok.src.to_string()),
        }
    }
}

impl<'a, E: Error + 'static> std::fmt::Display for Trace<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err.to_string())
    }
}

impl<'a> From<Trace<'a, LxErr>> for Trace<'a, SynErr> {
    fn from(trc: Trace<'a, LxErr>) -> Self {
        Trace {
            src: trc.src,
            err: SynErr::from(trc.err),
        }
    }
}
