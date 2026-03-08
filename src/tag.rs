use crate::{
    errors::BitError,
    object::{GitObject, ObjectType},
    util::parse_line,
};

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

    fn parse_body(body: &[u8]) -> Result<Self, BitError> {
        let (Some(object), rest) = parse_line(b"object ", body) else {
            return Err(BitError::InvalidTag("Missing object".into()));
        };

        let (Some(type_), rest) = parse_line(b"type ", rest) else {
            return Err(BitError::InvalidTag("Missing type".into()));
        };

        let (Some(tag), rest) = parse_line(b"tag ", rest) else {
            return Err(BitError::InvalidTag("Missing tag".into()));
        };

        let (Some(tagger), rest) = parse_line(b"tagger ", rest) else {
            return Err(BitError::InvalidTag("Missing tagger".into()));
        };

        let rest = rest
            .strip_prefix(b"\n")
            .ok_or_else(|| BitError::InvalidTag("Require empty line before message".into()))?;

        Ok(Self {
            object,
            type_: type_.parse()?,
            tag,
            tagger,
            message: String::from_utf8(rest.to_vec())?,
        })
    }
}
