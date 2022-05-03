use std::fmt::{Display, Formatter};
use pest::error::Error as PestError;
use pest::RuleType;
use serde::{Deserialize, Serialize};

// hint: generate `source` from `Span::as_str()`
// hint: Get the position with `Span::start() -> usize` and `Span::end() -> usize`
#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    // TODO: the error should contain the line and column number
    PestError { message: String },
    ParseError { message: String, source: String },
    EvaluateError { message: String, source: String },
    InternalError,
}

// TODO: JSON format
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::PestError { message } => write!(f, "Pest error: {}", message),
            Error::ParseError { message, source } => {
                write!(f, "Parse error: {}\n{}", message, source)
            },
            Error::EvaluateError { message, source } => {
                write!(f, "Evaluate error: {}\n{}", message, source)
            },
            Error::InternalError => write!(f, "Internal error"),
        }
    }
}

impl<R: RuleType> From<PestError<R>> for Error {
    fn from(error: PestError<R>) -> Self {
        Error::PestError {
            message: format!("{}", error),
        }
    }
}
