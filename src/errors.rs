
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidLine(String)
}

pub type ParseResult<T> = Result<T, ParseError>;