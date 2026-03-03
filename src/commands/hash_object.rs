use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use clap::Args;
use flate2::{Compression, write::ZlibEncoder};
use sha1::{Digest, Sha1};

use crate::{
    errors::BitError,
    object::ObjectType,
    util::{object_path, repo_root},
};

#[derive(Args, Debug)]
pub struct HashObjectArg {
    #[arg(short, long, default_value_t = ObjectType::Blob)]
    pub type_: ObjectType,

    #[arg(short, long, default_value_t = false)]
    pub write: bool,

    pub path: String,
}

impl HashObjectArg {
    pub fn run(self) -> Result<(), BitError> {
        let target_content = fs::read(&self.path)?;

        let object = crate::object::Object::new(self.type_, target_content);

        let object_output = object.serialize();
        let mut hasher = Sha1::new();
        hasher.update(&object_output);
        let hashed = hasher.finalize();
        let hash = format!("{:x}", hashed);

        if self.write {
            let path = object_path(repo_root()?, &hash);

            if !path.exists() {
                fs::create_dir_all(path.parent().expect("Could not get parent directory"))?;

                let file = OpenOptions::new().write(true).create_new(true).open(path)?;

                ZlibEncoder::new(file, Compression::default()).write_all(&object_output)?;
            }
        }

        println!("{}", hash);

        Ok(())
    }
}
