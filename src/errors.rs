use std::{io, str::Utf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitError {
    #[error("Invalid UTF-8 content")]
    InvalidUtf8(#[from] Utf8Error),

    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),

    #[error("{0}")]
    UnknownObjectType(#[from] crate::object::UnknownObjectTypeError),

    #[error("Not a bit repository")]
    NotInRepo,
}
