use std::{env, path::PathBuf};

use crate::errors::BitError;

pub fn repo_root() -> Result<PathBuf, BitError> {
    let mut cwd = env::current_dir()?;
    loop {
        if cwd.join(".bit").exists() {
            return Ok(cwd);
        }

        if let Some(next) = cwd.parent() {
            cwd = next.to_path_buf();
        } else {
            return Err(BitError::NotInRepo);
        }
    }
}

pub fn object_path(root: PathBuf, hash: &str) -> PathBuf {
    root.join(".bit/objects").join(&hash[..2]).join(&hash[2..])
}
