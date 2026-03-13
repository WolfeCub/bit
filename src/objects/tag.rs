use crate::{
    objects::{GitObject, ObjectType},
    utils::parse_field,
};

use anyhow::Context;

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
        let (Some(object), rest) = parse_field(b"object ", body) else {
            anyhow::bail!("Invalid tag: Missing object");
        };

        let (Some(type_), rest) = parse_field(b"type ", rest) else {
            anyhow::bail!("Invalid tag: Missing type");
        };

        let (Some(tag), rest) = parse_field(b"tag ", rest) else {
            anyhow::bail!("Invalid tag: Missing tag");
        };

        let (Some(tagger), rest) = parse_field(b"tagger ", rest) else {
            anyhow::bail!("Invalid tag: Missing tagger");
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
