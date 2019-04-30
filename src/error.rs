use serde::{de, ser};

use std::{error, fmt, io, result};

pub struct Error {
    err: Box<ErrorImpl>,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn classify(&self) -> Category {
        match self.err.code {
            ErrorCode::TupleHint(_, _) => Category::TupleHint,
            ErrorCode::Message(_) => Category::Data,
            ErrorCode::IO(_) => Category::IO,
            ErrorCode::NotImplemented => Category::Internal,
            ErrorCode::HexParsing(_) => Category::Syntax,
            ErrorCode::Parsing(_) => Category::Syntax,
        }
    }

    pub fn tuple_hint(&self) -> Option<TupleHint> {
        match self.err.code {
            ErrorCode::TupleHint(ref hint, _) => Some(*hint),
            _ => None,
        }
    }

    pub fn is_hint(&self) -> bool {
        self.classify() == Category::TupleHint
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
    TupleHint,
    IO,
    Syntax,
    Data,
    EOF,
    Internal,
}

#[derive(Clone, Copy)]
pub struct TupleHint {
    pub index: u64,
    pub is_dynamic: bool,
}

impl TupleHint {
    pub fn new(index: u64, is_dynamic: bool) -> Self {
        TupleHint { index, is_dynamic }
    }
}

struct ErrorImpl {
    code: ErrorCode,
}

pub enum ErrorCode {
    TupleHint(TupleHint, Error),
    Message(Box<str>),
    IO(io::Error),
    NotImplemented,
    HexParsing(hex::FromHexError),
    Parsing(Box<str>),
}

impl Error {
    pub(crate) fn message(s: &str) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::Message(s.to_string().into_boxed_str()),
            }),
        }
    }

    pub(crate) fn io(error: io::Error) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::IO(error),
            }),
        }
    }

    pub(crate) fn hint(hint: TupleHint, cause: Error) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::TupleHint(hint, cause),
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

    pub(crate) fn hex_parsing(error: hex::FromHexError) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::HexParsing(error),
            }),
        }
    }

    pub(crate) fn parsing(s: &str) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::Parsing(s.to_string().into_boxed_str()),
            }),
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::TupleHint(_, ref err) => fmt::Display::fmt(err, f),
            ErrorCode::Message(ref msg) => f.write_str(msg),
            ErrorCode::IO(ref err) => fmt::Display::fmt(err, f),
            ErrorCode::NotImplemented => f.write_str("not implemented"),
            ErrorCode::HexParsing(ref err) => fmt::Display::fmt(err, f),
            ErrorCode::Parsing(ref msg) => f.write_str(msg),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.err.code {
            ErrorCode::TupleHint(_, ref err) => error::Error::description(err),
            ErrorCode::IO(ref err) => error::Error::description(err),
            ErrorCode::Message(ref str) => str,
            ErrorCode::NotImplemented => "not implemented",
            ErrorCode::HexParsing(ref err) => error::Error::description(err),
            ErrorCode::Parsing(ref str) => str,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.err.code {
            ErrorCode::TupleHint(_, ref err) => Some(err),
            ErrorCode::IO(ref err) => Some(err),
            ErrorCode::HexParsing(ref err) => Some(err),
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
