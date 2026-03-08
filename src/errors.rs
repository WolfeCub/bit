use std::{borrow::Cow, io, path::StripPrefixError, str::Utf8Error, string::FromUtf8Error};

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
    InvalidCommit(Cow<'static, str>),

    #[error("Invalid tree: {0}")]
    InvalidTree(Cow<'static, str>),

    #[error("Invalid tree entry mode: {0}")]
    InvalidTreeEntryMode(String),

    #[error("Invalid tag: {0}")]
    InvalidTag(Cow<'static, str>),

    #[error("Empty {0} message")]
    EmptyMessage(Cow<'static, str>),

    #[error("Unable to strip prefix: {0}")]
    StripPrefix(#[from] StripPrefixError)
}
