#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LexError {
    #[error("L: {line} | C: {col} :: Invalid character :: {ch}")]
    InvalidCharacter { line: usize, col: usize, ch: char },

    #[error("L: {line} | C: {col} :: Invalid token :: {tok}")]
    InvalidToken {
        line: usize,
        col: usize,
        tok: String,
    },

    #[error("L: {line} | C: {col} :: Invalid insert keyword (expected `for` or valid identifier)")]
    InvalidInsertKeyword { line: usize, col: usize },

    #[error("L: {line} | C: {col} :: Unexpected EOF")]
    UnexpectedEOF { line: usize, col: usize },

    #[error("L: {line} | C: {col} :: Unterminated string")]
    UnterminatedString { line: usize, col: usize },

    #[error("L: {line} | C: {col} :: Unterminated insertion")]
    UnterminatedInsertion { line: usize, col: usize },

    #[error("L: {line} | C: {col} :: Empty insertion")]
    EmptyInsertion { line: usize, col: usize },

    #[error("L: {line} | C: {col} :: Expected {expected} :: {found}")]
    Expected {
        line: usize,
        col: usize,
        expected: String,
        found: String,
    },

    #[error("EOF")]
    EOF,

    #[error("Not an initializer")]
    NotInit,
}

#[derive(Debug, thiserror::Error, PartialEq)]
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

    #[error(transparent)]
    LexError(#[from] LexError),

    #[error("Unimplemented")]
    Unimplemented,
}
