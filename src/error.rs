use std::{fmt, io, result};

use failure::{Error, Fail};
use serde::{de, ser};


#[derive(Debug, Fail)]
pub enum SerdeEthError {
    #[fail(display = "invalid toolchain name: {}", cause)]
    WriteError{cause: io::Error},

    #[fail(display = "serialization error: {}", msg)]
    SerError{msg: String},

    #[fail(display = "not implemented")]
    NotImplemented
}

impl SerdeEthError {
    pub(crate) fn write_io(error: io::Error) -> Self {
        SerdeEthError::WriteError{cause: error}
    }
}

pub type Result<T> = result::Result<T, Error>;

impl ser::Error for SerdeEthError {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        SerdeEthError::SerError{msg: msg.to_string()}
    }
}
