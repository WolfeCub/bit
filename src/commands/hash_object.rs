use std::{
    fs::{self, OpenOptions},
    io::Write, path::Path,
};

use clap::Args;
use flate2::{Compression, write::ZlibEncoder};
use sha1::{Digest, Sha1};

use crate::{
    objects::{GitObject, Object, ObjectType}, utils::repo::object_path,
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
    pub fn run(self) -> anyhow::Result<()> {
        let hash = hash_object_hex_from_disk(&self.path, self.type_, self.write)?;

        println!("{}", hash);

        Ok(())
    }
}

pub fn hash_object<T: GitObject>(
    type_: ObjectType,
    inner: T,
    write: bool,
) -> anyhow::Result<[u8; 20]> {
    let object = Object::<T>::new(type_, inner);

    let object_output = object.serialize();
    let mut hasher = Sha1::new();
    hasher.update(&object_output);
    let hashed = hasher.finalize();
    let hash = format!("{:x}", hashed);

    if write {
        let path = object_path(&hash)?;

        if !path.exists() {
            fs::create_dir_all(path.parent().expect("Could not get parent directory"))?;

            let file = OpenOptions::new().write(true).create_new(true).open(path)?;

            ZlibEncoder::new(file, Compression::default()).write_all(&object_output)?;
        }
    }

    Ok(hashed.into())
}

pub fn hash_object_hex<T: GitObject>(
    type_: ObjectType,
    inner: T,
    write: bool,
) -> anyhow::Result<String> {
    let hash = hash_object(type_, inner, write)?;
    Ok(hex::encode(hash))
}

pub fn hash_object_hex_from_disk(path: impl AsRef<Path>, type_: ObjectType, write: bool) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    hash_object_hex(type_, content, write)
}

pub fn hash_object_from_disk(path: impl AsRef<Path>, type_: ObjectType, write: bool) -> anyhow::Result<[u8; 20]> {
    let content = fs::read(path)?;
    hash_object(type_, content, write)
}
