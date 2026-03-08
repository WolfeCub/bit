use chrono::{DateTime, FixedOffset};

use crate::{errors::BitError, object::GitObject, util::parse_line};

#[derive(Debug)]
pub struct Commit {
    pub hash: String,
    pub tree: String,
    pub parent: Option<String>,
    pub author: String,
    pub committer: String,
    pub gpgsig: Option<String>,
    pub message: String,
}

impl GitObject for Commit {
    fn serialize_body(&self) -> Vec<u8> {
        format!(
            "tree {}\n{}author {}\ncommitter {}\n{}{}",
            self.tree,
            self.parent
                .as_ref()
                .map_or("".to_string(), |p| format!("parent {}\n", p)),
            self.author,
            self.committer,
            self.gpgsig
                .as_ref()
                .map_or("".to_string(), |s| format!("gpgsig {}\n", s)),
            self.message
        )
        .into_bytes()
    }

    fn parse_body(hash: String, body: &[u8]) -> Result<Self, BitError> {
        let (Some(tree), rest) = parse_line(b"tree ", body) else {
            return Err(BitError::InvalidCommit("Missing tree".into()));
        };

        let (parent, rest) = parse_line(b"parent ", rest);

        let (Some(author), rest) = parse_line(b"author ", rest) else {
            return Err(BitError::InvalidCommit("Missing author".into()));
        };

        let (Some(committer), rest) = parse_line(b"committer ", rest) else {
            return Err(BitError::InvalidCommit("Missing committer".into()));
        };

        let (gpgsig, rest) = parse_line(b"gpgsig ", rest);

        let Some(rest) = rest.strip_prefix(b"\n") else {
            return Err(BitError::InvalidCommit(
                "Require empty line before body".into(),
            ));
        };

        Ok(Self {
            hash,
            tree,
            parent,
            author,
            committer,
            gpgsig,
            message: String::from_utf8(rest.to_vec())?,
        })
    }
}

impl Commit {
    pub fn parse_author_date(&self) -> (String, DateTime<FixedOffset>) {
        let split_idx = self
            .author
            .rmatch_indices(' ')
            .nth(1)
            .map(|(i, _)| i)
            .expect("Invalid author format");

        (
            self.author[..split_idx].to_string(),
            DateTime::parse_from_str(&self.author[split_idx..], "%s %z").unwrap(),
        )
    }
}
