use std::error::Error;

use crate::token::{Source, Token};

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
    #[error("Syn: Expected [{0}], found [{1}] :: {2}")]
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

// #[derive(Debug, thiserror::Error, PartialEq, Clone)]
// pub enum LexError {
//     #[error("L: {line} | C: {col} :: Invalid character :: {ch}")]
//     InvalidCharacter { line: usize, col: usize, ch: char },

//     #[error("L: {line} | C: {col} :: Invalid token :: {tok}")]
//     InvalidToken {
//         line: usize,
//         col: usize,
//         tok: String,
//     },

//     #[error("L: {line} | C: {col} :: Invalid insert keyword (expected `for` or valid identifier)")]
//     InvalidInsertKeyword { line: usize, col: usize },

//     #[error("L: {line} | C: {col} :: Invalid escape sequence :: {ch}")]
//     InvalidEscapeSequence { line: usize, col: usize, ch: char },

//     #[error("L: {line} | C: {col} :: Unexpected EOF")]
//     UnexpectedEOF { line: usize, col: usize },

//     #[error("L: {line} | C: {col} :: Unterminated string")]
//     UnterminatedString { line: usize, col: usize },

//     #[error("L: {line} | C: {col} :: Unterminated insertion")]
//     UnterminatedInsertion { line: usize, col: usize },

//     #[error("L: {line} | C: {col} :: Empty insertion")]
//     EmptyInsertion { line: usize, col: usize },

//     #[error("L: {line} | C: {col} :: Expected {expected} :: {found}")]
//     Expected {
//         line: usize,
//         col: usize,
//         expected: String,
//         found: String,
//     },

//     #[error("EOF")]
//     EOF,

//     #[error("Not an initializer")]
//     NotInit,
// }

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ParseError {
    #[error("L: {line} | C: {col} :: Expected {expected} :: {found}")]
    Expected {
        line: usize,
        col: usize,
        expected: String,
        found: String,
    },

    #[error("L: {line} | C: {col} :: Invalid type :: {ty}")]
    InvalidType { line: usize, col: usize, ty: String },

    #[error("L: {line} | C: {col} :: Invalid token :: {tok}")]
    InvalidToken {
        line: usize,
        col: usize,
        tok: String,
    },

    #[error("L: {line} | C: {col} :: Unexpected EOF")]
    UnexpectedEOF { line: usize, col: usize },

    #[error("No top-level expressions allowed.")]
    NoTopLevelExpressionsAllowed,

    #[error(transparent)]
    LexError(#[from] LxErr),

    #[error("Unimplemented")]
    Unimplemented,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum SemanticError {
    #[error("Conflict: An element with this name already exists :: {0}")]
    AlreadyExists(String),

    #[error("Invalid type :: {0}")]
    InvalidType(String),

    #[error("Unknown type :: {0}")]
    UnknownType(String),

    #[error("Unknown variable :: {0}")]
    UnknownVariable(String),

    #[error("Unknown fmt :: {0}")]
    UnknownFmt(String),

    #[error("Unused Insertion, Should be `str` :: {0}")]
    UnusedInsertion(String),

    #[error("Out of scope :: {0}. Expected :: {1}, Found :: {2}")]
    OutOfScope(String, String, String),

    #[error("Required fields must be declared in the beginning of the file :: {0}")]
    RequiredsOnlyAtTop(String),

    #[error("No main struct found")]
    NoMainStruct,

    #[error(transparent)]
    ParseError(#[from] ParseError),
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
        let red = "\x1b[31m";
        let green = "\x1b[32m";
        let reset = "\x1b[0m";

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
                line.push_str(red);
            } else {
                line.push_str(green);
            }
            line.push(src[i] as char);
            if i == self.src.bix {
                line.push_str(reset);
            }
        }
        line.push_str(reset);

        // line.push_str(&std::str::from_utf8(&src[start_ix..end_ix]).unwrap());

        line.push('\n');

        for i in 0..line.len() {
            if i < self.src.bix - start_ix {
                line.push(' ');
            } else if i >= self.src.bix - start_ix && i < self.src.bix - start_ix + self.src.len {
                line.push_str(red);
                line.push('^');
                line.push_str(reset);
            } else {
                break;
            }
        }

        line
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

impl<'a> From<Trace<'a, LxErr>> for Trace<'a, ParseError> {
    fn from(trc: Trace<'a, LxErr>) -> Self {
        Trace {
            src: trc.src,
            err: ParseError::from(trc.err),
        }
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
