use chrono::{DateTime, FixedOffset};

use crate::{objects::GitObject, utils::parse_field};

#[derive(Debug)]
pub struct Commit {
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
            "tree {}\n{}author {}\ncommitter {}\n{}\n{}\n",
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

    fn parse_body(body: &[u8]) -> anyhow::Result<Self> {
        let (Some(tree), rest) = parse_field(b"tree ", body) else {
            anyhow::bail!("Invalid commit: Missing tree");
        };

        let (parent, rest) = parse_field(b"parent ", rest);

        let (Some(author), rest) = parse_field(b"author ", rest) else {
            anyhow::bail!("Invalid commit: Missing author");
        };

        let (Some(committer), rest) = parse_field(b"committer ", rest) else {
            anyhow::bail!("Invalid commit: Missing committer");
        };

        let (gpgsig, rest) = parse_field(b"gpgsig ", rest);

        let Some(rest) = rest.strip_prefix(b"\n") else {
            anyhow::bail!("Invalid commit: Require empty line before body");
        };

        Ok(Self {
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
