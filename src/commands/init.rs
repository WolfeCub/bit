use std::{fs, path};

use clap::Args;

use crate::errors::BitError;

#[derive(Args, Debug)]
pub struct InitArg {
    pub path: Option<String>,
}

impl InitArg {
    pub fn run(self) -> Result<(), BitError> {
        if let Some(path) = self.path.as_ref() {
            fs::create_dir(path)?;
        }

        let path = format!("{}/.bit", self.path.unwrap_or_else(|| ".".to_string()));
        fs::create_dir(&path)?;

        let absolute = path::absolute(path)?;
        println!("Initialized empty Git repository in {}", absolute.display());

        Ok(())
    }
}
