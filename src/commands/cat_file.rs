use clap::Args;

use crate::{
    objects::{Object, ObjectType},
};

#[derive(Args, Debug)]
pub struct CatFileArg {
    pub type_: ObjectType,
    pub object: String,
}

impl CatFileArg {
    pub fn run(self) -> anyhow::Result<()> {
        let object = Object::<Vec<u8>>::read_from_disk(&self.object, self.type_)?;

        println!("{}", unsafe { str::from_utf8_unchecked(&object.inner) });

        Ok(())
    }
}
