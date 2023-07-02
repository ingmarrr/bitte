pub type TraceResult<T> = Result<T, TraceError>;
pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("expected {expected:?}, got {got:?}")]
    Expected { expected: String, got: String },
}

#[derive(Debug, thiserror::Error)]
pub struct TraceError {
    pub error: ParseError,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = self.line + 1;
        let column = self.column + 1;
        f.write_fmt(format_args!(
            "Error at line {}, column {}: {}",
            line, column, self.error
        ))
    }
}
