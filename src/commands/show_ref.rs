use clap::Args;

use crate::errors::BitError;

#[derive(Args, Debug)]
pub struct ShowRefArg {
}

impl ShowRefArg {
    pub fn run(self) -> Result<(), BitError> {

        Ok(())
    }
}

