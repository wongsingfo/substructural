use pest::error::Error as PestError;
use pest::RuleType;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

// hint: generate `source` from `Span::as_str()`
// hint: Get the position with `Span::start() -> usize` and `Span::end() -> usize`
#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    ParseError {
        message: String,
        start: usize,
        end: usize,
    },
    EvaluateError {
        message: String,
        source: String,
    },
    TypeError {
        message: String,
        start: usize,
        end: usize,
    },
    InternalError {
        message: String,
    },
}

// TODO: JSON format
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::ParseError { message, .. } => {
                write!(f, "Parse error: {}\n", message)
            }
            Error::EvaluateError { message, source } => {
                write!(f, "Evaluate error: {}\n{}", message, source)
            }
            Error::TypeError {
                message,
                start,
                end,
            } => {
                write!(f, "Type error in [Ln {}, Col {}]: {}\n", start, end, message)
            }
            Error::InternalError {
                message,
            } => write!(f, "Internal error: {}", message),
        }
    }
}

impl<R: RuleType> From<PestError<R>> for Error {
    fn from(error: PestError<R>) -> Self {
        let location = &error.location;
        let start = match location {
            pest::error::InputLocation::Pos(i) => *i,
            pest::error::InputLocation::Span((i, _)) => *i,
        };
        let end = match location {
            pest::error::InputLocation::Pos(_) => start + 1,
            pest::error::InputLocation::Span((_, j)) => *j,
        };
        Error::ParseError {
            message: format!("{}", error),
            start,
            end,
        }
    }
}
