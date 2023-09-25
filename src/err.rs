#[derive(Debug, thiserror::Error, PartialEq, Clone)]
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

    #[error("L: {line} | C: {col} :: Invalid escape sequence :: {ch}")]
    InvalidEscapeSequence { line: usize, col: usize, ch: char },

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
    LexError(#[from] LexError),

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

    #[error("Unknown fmt :: {0}")]
    UnknownFmt(String),

    #[error("Unused Insertion, Should be `str` :: {0}")]
    UnusedInsertion(String),

    #[error("Out of scope :: {0}. Expected :: {1}, Found :: {2}")]
    OutOfScope(String, String, String),

    #[error("Required fields must be declared in the beginning of the file :: {0}")]
    RequiredsOnlyAtTop(String),

    #[error(transparent)]
    ParseError(#[from] ParseError),
}
