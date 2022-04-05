use pest::error::Error as PestError;
use pest::RuleType;

#[derive(Debug)]
pub enum Error {
    PestError { message: String },
    ParseError { message: String, source: String },
    InternalError,
}

impl<R: RuleType> From<PestError<R>> for Error {
    fn from(error: PestError<R>) -> Self {
        Error::PestError {
            message: format!("{}", error),
        }
    }
}
