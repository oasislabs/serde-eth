use std::{error, fmt, io, result};

use serde::{de, ser};

pub struct Error {
    err: Box<ErrorImpl>,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn classify(&self) -> Category {
        match self.err.code {
            ErrorCode::Message(_) => Category::Data,
            ErrorCode::IO(_) => Category::IO,
            ErrorCode::NotImplemented => Category::Internal,
        }
    }

    pub fn is_io(&self) -> bool {
        self.classify() == Category::IO
    }

    pub fn is_syntax(&self) -> bool {
        self.classify() == Category::Syntax
    }

    pub fn is_data(&self) -> bool {
        self.classify() == Category::Data
    }

    pub fn is_eof(&self) -> bool {
        self.classify() == Category::EOF
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Category {
    IO,
    Syntax,
    Data,
    EOF,
    Internal,
}

struct ErrorImpl {
    code: ErrorCode,
}

pub enum ErrorCode {
    Message(Box<str>),
    IO(io::Error),
    NotImplemented,
}

impl Error {
    pub(crate) fn io(error: io::Error) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::IO(error),
            }),
        }
    }

    pub(crate) fn not_implemented() -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::NotImplemented,
            }),
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Message(ref msg) => f.write_str(msg),
            ErrorCode::IO(ref err) => fmt::Display::fmt(err, f),
            ErrorCode::NotImplemented => f.write_str("not implemented"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.err.code {
            ErrorCode::IO(ref err) => error::Error::description(err),
            ErrorCode::Message(ref str) => str,
            ErrorCode::NotImplemented => "not implemented",
            _ => "Eth parse error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.err.code {
            ErrorCode::IO(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.err, f)
    }
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.code, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {:?}", self.err.code.to_string(),)
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        make_error(msg.to_string())
    }

    fn invalid_type(unexp: de::Unexpected, exp: &de::Expected) -> Self {
        if let de::Unexpected::Unit = unexp {
            Error::custom(format_args!("invalid type: null, expected {}", exp))
        } else {
            Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
        }
    }
}

impl ser::Error for Error {
    #[cold]
    fn custom<T: fmt::Display>(msg: T) -> Error {
        make_error(msg.to_string())
    }
}

// Parse our own error message that looks like "{} at line {} column {}" to work
// around erased-serde round-tripping the error through de::Error::custom.
fn make_error(msg: String) -> Error {
    Error {
        err: Box::new(ErrorImpl {
            code: ErrorCode::Message(msg.into_boxed_str()),
        }),
    }
}
