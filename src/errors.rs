use std::{io, str::Utf8Error, string::FromUtf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitError {
    #[error("Invalid UTF-8 content")]
    InvalidUtf8(#[from] Utf8Error),

    #[error("Invalid UTF-8 content")]
    InvalidFromUtf8(#[from] FromUtf8Error),

    #[error("IO Error: {0}")]
    IOError(#[from] io::Error),

    #[error("{0}")]
    UnknownObjectType(#[from] crate::object::UnknownObjectTypeError),

    #[error("Not a bit repository")]
    NotInRepo,

    #[error("Invalid commit: {0}")]
    InvalidCommit(String),

    #[error("Invalid tree: {0}")]
    InvalidTree(String),

    #[error("Invalid tree entry mode: {0}")]
    InvalidTreeEntryMode(String),
}
