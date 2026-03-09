use clap::Args;

use crate::{errors::BitError, util::find_hash};

#[derive(Args, Debug)]
pub struct RevParseArg {
    pub hash_or_ref: Option<String>,
}

impl RevParseArg {
    pub fn run(self) -> Result<(), BitError> {
        let Some(hash_or_ref) = self.hash_or_ref else {
            return Ok(());
        };

        let hash = find_hash(&hash_or_ref)?;
        println!("{}", hash);
        Ok(())
    }
}
