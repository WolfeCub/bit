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
        // TODO: We should actually use the type to resolve the object. 
        // i.e. if you specify tree and give a commit hash we can figure out the tree hash from the
        //      commit and then print it not just error out.
        let object = Object::<String>::read_from_disk(&self.object, self.type_)?;

        println!("{}", object.inner);

        Ok(())
    }
}
