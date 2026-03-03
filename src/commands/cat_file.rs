use clap::Args;

use crate::{errors::BitError, object::Object};

#[derive(Args, Debug)]
pub struct CatFileArg {
    pub type_: String,
    pub object: String,
}

impl CatFileArg {
    pub fn run(self) -> Result<(), BitError> {
        let object = Object::read_from_disk(self.type_, self.object);

        println!("{}", str::from_utf8(&object.content)?);

        Ok(())
    }
}
