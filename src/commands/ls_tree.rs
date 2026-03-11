use clap::Args;

use crate::{
    objects::{Object, ObjectType, Tree},
};

#[derive(Args, Debug)]
pub struct LsTreeArg {
    pub hash: String,
}

impl LsTreeArg {
    pub fn run(self) -> anyhow::Result<()> {
        let tree = Object::<Tree>::read_from_disk(&self.hash, ObjectType::Tree)?;

        for entry in tree.inner.entries {
            println!(
                "{:0>6} {} {}\t{}",
                entry.mode,
                entry.get_type()?,
                entry.hash,
                entry.path
            );
        }

        Ok(())
    }
}
