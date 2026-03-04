use std::io::BufRead;

use itertools::Itertools;

use crate::errors::BitError;

#[derive(Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn parse(body: &[u8]) -> Result<Self, BitError> {
        let mut entries = Vec::new();
        let mut iter = body;
        loop {
            if iter.is_empty() {
                break;
            }

            let (entry, rest) = TreeEntry::parse(iter)?;
            entries.push(entry);

            iter = rest;
        }

        Ok(Tree { entries })
    }
}

#[derive(Debug)]
pub struct TreeEntry {
    pub mode: String,
    pub path: String,
    pub hash: String,
}

impl TreeEntry {
    pub fn parse(line: &[u8]) -> Result<(Self, &[u8]), BitError> {
        let (mode, rest) = line.splitn(2, |c| *c == b' ').next_tuple().ok_or_else(|| {
            BitError::InvalidTree("Invalid tree entry. No space found.".to_string())
        })?;

        let (path, rest) = rest
            .splitn(2, |c| *c == b'\0')
            .next_tuple()
            .ok_or_else(|| {
                BitError::InvalidTree("Invalid tree entry. No null byte found.".to_string())
            })?;

        let hash = hex::encode(&rest[..20]);

        Ok((
            TreeEntry {
                mode: String::from_utf8(mode.to_vec())?,
                path: String::from_utf8(path.to_vec())?,
                hash,
            },
            &rest[20..],
        ))
    }

    pub fn get_type(&self) -> Result<&'static str, BitError> {
        let mode = u32::from_str_radix(&self.mode, 8)
            .map_err(|_| BitError::InvalidTreeEntryMode(self.mode.clone()))?;

        match mode >> 12 {
            0o04 => Ok("tree"),
            0o10 => Ok("blob"),
            0o12 => Ok("blob"),
            0o16 => Ok("commit"),
            _ => Err(BitError::InvalidTreeEntryMode(self.mode.clone())),
        }
    }
}
