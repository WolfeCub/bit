use std::{
    fs,
    io::{self, Read},
    str::FromStr,
};

use flate2::read::ZlibDecoder;

use crate::util::repo_root;

pub struct Object {
    pub type_: ObjectType,
    pub length: usize,
    pub content: Vec<u8>,
}

impl Object {
    pub fn new(type_: ObjectType, content: Vec<u8>) -> Self {
        Object {
            type_,
            length: content.len(),
            content,
        }
    }

    pub fn output(&self) -> Vec<u8> {
        [
            self.type_.to_string().as_bytes(),
            b" ",
            self.length.to_string().as_bytes(),
            b"\0",
            &self.content,
        ]
        .concat()
    }

    pub fn read_from_disk(type_: String, hash: String) -> Self {
        let path = repo_root()
            .expect("Not in bit repository")
            .join(".bit/objects")
            .join(&hash[..2])
            .join(&hash[2..]);

        // TODO: buffer this reading and decompressing
        let compressed_contents =
            fs::read(&path).expect(&format!("Unable to read {}", path.display()));
        let mut d = ZlibDecoder::new(io::Cursor::new(compressed_contents));
        let mut contents = vec![];
        d.read_to_end(&mut contents)
            .expect("Unable to read compressed contents");

        // TODO: Size not needed?
        let Some((rest, size)) = read_header(&type_, contents.as_slice()) else {
            panic!("fatal: bit cat-file {}: bad file", hash);
        };

        Object {
            type_: ObjectType::from_str(&type_).expect(&format!("Unknown object type: {}", type_)),
            length: size,
            content: rest.to_vec(),
        }
    }
}

#[derive(Debug)]
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

// TODO: Elegant error handling
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
