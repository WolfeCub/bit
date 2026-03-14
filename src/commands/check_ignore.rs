use std::path::Path;

use clap::Args;

use crate::objects::Ignore;

/// Checks if the passed path is ignored by .bitignore and prints it if it is
#[derive(Args, Debug)]
pub struct CheckIgnoreArg {
    pub path: String,
}

impl CheckIgnoreArg {
    pub fn run(self) -> anyhow::Result<()> {
        let bitignore = Ignore::build_from_disk()?;

        let is_dir = Path::new(&self.path).is_dir();
        let ignored = bitignore.is_file_ignored(&self.path, is_dir);

        if ignored {
            println!("{}", self.path);
        }

        Ok(())
    }
}

