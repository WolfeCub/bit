use std::{
    fs,
    io::{self, Read},
    str::FromStr,
};

use flate2::read::ZlibDecoder;

use crate::{errors::BitError, util::repo_root};

pub trait GitObject: Sized {
    fn serialize_body(&self) -> Vec<u8>;
    // TODO: Remove hash from parse_body it's just there for Commit
    fn parse_body(hash: String, body: &[u8]) -> Result<Self, BitError>;
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
            self.type_.to_string().as_bytes(),
            b" ",
            body.len().to_string().as_bytes(),
            b"\0",
            &body,
        ]
        .concat()
    }

    pub fn read_from_disk(hash: &str, type_: ObjectType) -> Result<Self, BitError> {
        let path = repo_root()?
            .join(".bit/objects")
            .join(&hash[..2])
            .join(&hash[2..]);

        // TODO: buffer this reading and decompressing
        let compressed_contents = fs::read(&path)?;
        let mut d = ZlibDecoder::new(io::Cursor::new(compressed_contents));
        let mut contents = vec![];
        d.read_to_end(&mut contents)?;

        // TODO: Size not needed?
        let Some((rest, _size)) = read_header(&type_.to_string(), contents.as_slice()) else {
            panic!("fatal: bit cat-file {}: bad file", hash);
        };

        Ok(Object {
            inner: T::parse_body(hash.to_string(), rest)?,
            type_,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

#[derive(thiserror::Error, Debug)]
#[error("Unknown object type: {0}")]
pub struct UnknownObjectTypeError(String);

impl FromStr for ObjectType {
    type Err = UnknownObjectTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            t => Err(UnknownObjectTypeError(t.to_string())),
        }
    }
}

// TODO: Maybe don't allocate here
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

    fn parse_body(_hash: String, body: &[u8]) -> Result<Self, BitError> {
        Ok(body.to_vec())
    }
}
