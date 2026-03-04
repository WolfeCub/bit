use clap::Args;

use crate::{errors::BitError, object::{Object, ObjectType}, tree::Tree};

#[derive(Args, Debug)]
pub struct LsTreeArg {
    pub hash: String,
}

impl LsTreeArg {
    pub fn run(self) -> Result<(), BitError> {
        let tree = Object::<Tree>::read_from_disk(&self.hash, ObjectType::Tree)?;

        for entry in tree.inner.entries {
            println!("{:0>6} {} {}\t{}", entry.mode, entry.get_type()?, entry.hash, entry.path);
        }

        Ok(())
    }
}

