use std::error::Error;

use crate::token::Source;

pub enum LxErrKind {
    InvalidToken,
    InvalidCharacter,
    InvalidUtf8,
    UnexpectedEOF,
    UnterminatedString,
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum LxErr {
    #[error("Invalid token :: {0}")]
    InvalidToken(String),
    #[error("Invalid character :: {0}")]
    InvalidCharacter(String),
    #[error("Invalid Utf8 :: {0}")]
    InvalidUtf8(String),
    #[error("Unexpected EOF :: {0}")]
    UnexpectedEOF(String),
    #[error("Unterminated String :: {0}")]
    Unterminated(String),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, thiserror::Error)]
pub enum SynErr {}

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
    pub fn new(src: Source, err: E) -> Self {
        Self { src, err }
    }
}

impl<'a> From<Trace<'a, LxErr>> for Trace<'a, ParseError> {
    fn from(trc: Trace<LxErr>) -> Self {
        Trace {
            src: trc.src,
            err: ParseError::from(trc.err),
        }
    }
}
