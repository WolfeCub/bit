use crate::{
    object::{GitObject, ObjectType},
    util::parse_line,
};

use anyhow::{Context, anyhow};

#[derive(Debug)]
pub struct Tag {
    pub object: String,
    pub type_: ObjectType,
    pub tag: String,
    pub tagger: String,
    pub message: String,
}

impl GitObject for Tag {
    fn serialize_body(&self) -> Vec<u8> {
        format!(
            "object {}\ntype {}\ntag {}\ntagger {}\n\n{}",
            self.object,
            Into::<&'static str>::into(self.type_),
            self.tag,
            self.tagger,
            self.message
        )
        .into_bytes()
    }

    fn parse_body(body: &[u8]) -> anyhow::Result<Self> {
        let (Some(object), rest) = parse_line(b"object ", body) else {
            return Err(anyhow!("Invalid tag: Missing object"));
        };

        let (Some(type_), rest) = parse_line(b"type ", rest) else {
            return Err(anyhow!("Invalid tag: Missing type"));
        };

        let (Some(tag), rest) = parse_line(b"tag ", rest) else {
            return Err(anyhow!("Invalid tag: Missing tag"));
        };

        let (Some(tagger), rest) = parse_line(b"tagger ", rest) else {
            return Err(anyhow!("Invalid tag: Missing tagger"));
        };

        let rest = rest
            .strip_prefix(b"\n")
            .context("Require empty line before message")?;

        Ok(Self {
            object,
            type_: type_.parse()?,
            tag,
            tagger,
            message: String::from_utf8(rest.to_vec())?,
        })
    }
}
