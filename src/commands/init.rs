use std::{fs, path};

use clap::Args;

#[derive(Args, Debug)]
pub struct InitArg {
    pub path: Option<String>,
}

impl InitArg {
    pub fn run(self) -> anyhow::Result<()> {
        if let Some(path) = self.path.as_ref() {
            fs::create_dir(path)?;
        }

        let path = format!("{}/.bit", self.path.unwrap_or_else(|| ".".to_string()));
        fs::create_dir(&path)?;

        fs::write(format!("{path}/HEAD"), "ref: refs/heads/master")?;

        let absolute = path::absolute(path)?;
        println!("Initialized empty Git repository in {}", absolute.display());

        Ok(())
    }
}
