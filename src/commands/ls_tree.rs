use std::{fs, path};

use clap::Args;

use crate::{errors::BitError, object::{Object, ObjectType}, tree::Tree};

#[derive(Args, Debug)]
pub struct LsTreeArg {
    pub hash: String,
}

impl LsTreeArg {
    pub fn run(self) -> Result<(), BitError> {
        let object = Object::read_from_disk(ObjectType::Tree, &self.hash);
        let tree = Tree::parse(&object.content)?;

        for entry in tree.entries {
            println!("{:0>6} {} {}\t{}", entry.mode, entry.get_type()?, entry.hash, entry.path);
        }

        Ok(())
    }
}

