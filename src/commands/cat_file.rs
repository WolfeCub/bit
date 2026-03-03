use clap::Args;

use crate::{errors::BitError, object::{Object, ObjectType}};

#[derive(Args, Debug)]
pub struct CatFileArg {
    pub type_: ObjectType,
    pub object: String,
}

impl CatFileArg {
    pub fn run(self) -> Result<(), BitError> {
        let object = Object::read_from_disk(self.type_, &self.object);

        println!("{}", str::from_utf8(&object.content)?);

        Ok(())
    }
}
