use std::{
    fs,
    io::{self, Read},
};

use clap::Args;
use flate2::read::ZlibDecoder;

use crate::util::repo_root;

#[derive(Args, Debug)]
pub struct CatFileArg {
    pub type_: String,
    pub object: String,
}

impl CatFileArg {
    pub fn run(self) {
        let path = repo_root()
            .expect("Not in bit repository")
            .join(".bit/objects")
            .join(&self.object[..2])
            .join(&self.object[2..]);

        // TODO: buffer this reading and decompressing
        let compressed_contents =
            fs::read(&path).expect(&format!("Unable to read {}", path.display()));
        let mut d = ZlibDecoder::new(io::Cursor::new(compressed_contents));
        let mut contents = vec![];
        d.read_to_end(&mut contents)
            .expect("Unable to read compressed contents");

        // TODO: Size not needed?
        let Some((rest, size)) = read_header(self.type_, contents.as_slice()) else {
            eprintln!("fatal: bit cat-file {}: bad file", self.object);
            return;
        };

        println!("{}", str::from_utf8(rest).expect("Invalid UTF-8 content"));
    }
}

// TODO: Elegant error handling
fn read_header(type_: String, contents: &[u8]) -> Option<(&[u8], usize)> {
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
