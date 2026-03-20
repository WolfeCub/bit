use clap::Args;

use crate::{
    objects::{Object, ObjectType},
};

/// Displays the contents of a bit object
#[derive(Args, Debug)]
pub struct CatFileArg {
    pub type_: ObjectType,
    pub object: String,
}

impl CatFileArg {
    pub fn run(self) -> anyhow::Result<()> {
        let object = Object::<String>::read_from_disk(&self.object, self.type_)?;

        println!("{}", object.inner);

        Ok(())
    }
}
