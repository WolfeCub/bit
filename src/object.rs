use std::{
    fs,
    io::{BufReader, Read},
    str::FromStr,
};

use flate2::bufread::ZlibDecoder;
use anyhow::anyhow;

use crate::{
    util::object_path,
};

pub trait GitObject: Sized {
    fn serialize_body(&self) -> Vec<u8>;
    fn parse_body(body: &[u8]) -> anyhow::Result<Self>;
}

pub struct Object<T: GitObject> {
    pub inner: T,
    pub type_: ObjectType,
}

impl<T: GitObject> Object<T> {
    pub fn new(type_: ObjectType, inner: T) -> Self {
        Object { type_, inner }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let body = self.inner.serialize_body();
        [
            Into::<&'static str>::into(self.type_).as_bytes(),
            b" ",
            body.len().to_string().as_bytes(),
            b"\0",
            &body,
        ]
        .concat()
    }

    pub fn read_from_disk(hash: &str, type_: ObjectType) -> anyhow::Result<Self> {
        let path = object_path(hash)?;

        let file_buf_reader = BufReader::new(fs::File::open(&path)?);
        let mut buf_decompressor = ZlibDecoder::new(file_buf_reader);

        let mut contents = vec![];
        buf_decompressor.read_to_end(&mut contents)?;

        // TODO: Size not needed?
        let Some((rest, _size)) = read_header(type_.into(), contents.as_slice()) else {
            panic!("fatal: bit cat-file {}: bad file", hash);
        };

        Ok(Object {
            inner: T::parse_body(rest)?,
            type_,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl FromStr for ObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            t => Err(anyhow!("Unknown object type: {}", t)),
        }
    }
}

impl From<ObjectType> for &'static str {
    fn from(type_: ObjectType) -> Self {
        match type_ {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
            ObjectType::Tag => "tag",
        }
    }
}

impl ToString for ObjectType {
    fn to_string(&self) -> String {
        match self {
            ObjectType::Blob => "blob".to_string(),
            ObjectType::Tree => "tree".to_string(),
            ObjectType::Commit => "commit".to_string(),
            ObjectType::Tag => "tag".to_string(),
        }
    }
}

fn read_header<'a>(type_: &str, contents: &'a [u8]) -> Option<(&'a [u8], usize)> {
    let rest = contents
        .strip_prefix(type_.as_bytes())?
        .strip_prefix(b" ")?;

    let pos = rest.iter().position(|&b| b == b'\0')?;
    let size = parse_number_from_bytes(&rest[..pos])?;
    // Skip \0
    let rest = &rest[pos + 1..];

    Some((rest, size))
}

fn parse_number_from_bytes(bytes: &[u8]) -> Option<usize> {
    let mut n: usize = 0;
    for &b in bytes {
        match b {
            b'0'..=b'9' => n = n * 10 + (b - b'0') as usize,
            _ => break,
        }
    }
    Some(n)
}

// Nice little convienence wrapper for a generic body of bytes.
impl GitObject for Vec<u8> {
    fn serialize_body(&self) -> Vec<u8> {
        self.clone()
    }

    fn parse_body(body: &[u8]) -> anyhow::Result<Self> {
        Ok(body.to_vec())
    }
}
