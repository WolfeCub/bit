use std::{
    fs::{self, OpenOptions},
    io::Write,
    str::FromStr,
};

use clap::Args;
use flate2::{
    Compression,
    write::ZlibEncoder,
};
use sha1::{Digest, Sha1};

use crate::{
    object::ObjectType,
    util::{object_path, repo_root},
};

#[derive(Args, Debug)]
pub struct HashObjectArg {
    #[arg(short, long)]
    pub type_: String,

    #[arg(short, long, default_value_t = false)]
    pub write: bool,

    pub path: String,
}

impl HashObjectArg {
    pub fn run(self) {
        let target_content = fs::read(&self.path).expect(&format!("Unable to read {}", self.path));

        let object_type = ObjectType::from_str(&self.type_)
            .expect(&format!("Unknown object type: {}", self.type_));

        let object = crate::object::Object::new(object_type, target_content);

        let object_output = object.output();
        let mut hasher = Sha1::new();
        hasher.update(&object_output);
        let hashed = hasher.finalize();
        let hash = format!("{:x}", hashed);

        if self.write {
            let root = repo_root().expect("Not in bit repository");
            let path = object_path(root, &hash);

            if !path.exists() {
                fs::create_dir_all(path.parent().expect("Could not get parent directory"))
                    .expect("Unable to create object directory");

                let Ok(file) = OpenOptions::new().write(true).create_new(true).open(path) else {
                    panic!("Unable to create object file");
                };

                ZlibEncoder::new(file, Compression::default())
                    .write_all(&object_output)
                    .expect("Unable to write compressed object");
            }
        }

        println!("{}", hash);
    }
}
