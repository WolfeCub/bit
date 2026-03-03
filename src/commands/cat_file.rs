use clap::Args;

use crate::object::Object;

#[derive(Args, Debug)]
pub struct CatFileArg {
    pub type_: String,
    pub object: String,
}

impl CatFileArg {
    pub fn run(self) {
        let object = Object::read_from_disk(self.type_, self.object);

        println!(
            "{}",
            str::from_utf8(&object.content).expect("Invalid UTF-8 content")
        );
    }
}
