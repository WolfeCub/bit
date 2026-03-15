use std::collections::HashMap;

use anyhow::Context;
use anyhow::anyhow;
use itertools::Itertools;

use crate::objects::Object;
use crate::objects::{GitObject, ObjectType};

#[derive(Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl GitObject for Tree {
    fn serialize_body(&self) -> Vec<u8> {
        self.entries
            .iter()
            .flat_map(|e| e.serialize())
            .collect::<Vec<u8>>()
    }

    fn parse_body(body: &[u8]) -> anyhow::Result<Self> {
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
    pub fn parse(line: &[u8]) -> anyhow::Result<(Self, &[u8])> {
        let (mode, rest) = line
            .splitn(2, |c| *c == b' ')
            .next_tuple()
            .context("Invalid tree entry. No space found.")?;

        let (path, rest) = rest
            .splitn(2, |c| *c == b'\0')
            .next_tuple()
            .context("Invalid tree entry. No null byte found.")?;

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

    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.mode.len() + 1 + self.path.len() + 1 + 20);

        out.extend_from_slice(self.mode.as_bytes());
        out.push(b' ');
        out.extend_from_slice(self.path.as_bytes());
        out.push(0);
        out.extend_from_slice(&hex::decode(&self.hash).expect("Incorrectly constructed hash"));

        out
    }

    pub fn get_type(&self) -> anyhow::Result<&'static str> {
        let mode = u32::from_str_radix(&self.mode, 8)
            .with_context(|| format!("Invalid tree entry mode: {}", self.mode))?;

        match mode >> 12 {
            0o04 => Ok("tree"),
            0o10 => Ok("blob"),
            0o12 => Ok("blob"),
            0o16 => Ok("commit"),
            _ => Err(anyhow!("Invalid tree entry mode: {}", self.mode)),
        }
    }
}

pub fn flatten_tree_from_disk(tree_hash: &str) -> anyhow::Result<HashMap<String, TreeEntry>> {
    pub fn helper_rec(
        tree_hash: &str,
        prefix_dir: &str,
    ) -> anyhow::Result<HashMap<String, TreeEntry>> {
        let mut map = HashMap::new();
        let tree = Object::<Tree>::read_from_disk(tree_hash, ObjectType::Tree)?;

        for entry in tree.inner.entries {
            let prefixed_path = format!("{prefix_dir}{}", entry.path);

            if entry.get_type()? == "blob" {
                map.insert(prefixed_path, entry);
            } else if entry.get_type()? == "tree" {
                let dir_path = format!("{}/", prefixed_path);
                map.extend(helper_rec(&entry.hash, &dir_path)?);
            }
        }

        Ok(map)
    }

    helper_rec(tree_hash, "")
}
