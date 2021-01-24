/// Desktop file parsing error.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidLine(String)
}

/// Result of the desktop file parser.
pub type ParseResult<T> = Result<T, ParseError>;