use std::{fs, path::Path};

use clap::Args;

use crate::errors::BitError;

#[derive(Args, Debug)]
pub struct ShowRefArg {}

impl ShowRefArg {
    pub fn run(self) -> Result<(), BitError> {
        let mut refs = list_ref_paths(".bit/refs")?;
        refs.sort();

        for r in refs.iter() {
            let hash = resolve_ref_path(r)?;
            let path = r
                .strip_prefix(".bit/")
                .expect("Somehow ref doesn't start with .bit/");
            println!("{} {}", hash, path);
        }
        Ok(())
    }
}

pub fn resolve_ref(reference: &str) -> Result<String, BitError> {
    resolve_ref_path(&format!(".bit/{}", reference))
}

fn resolve_ref_path(path: impl AsRef<Path>) -> Result<String, BitError> {
    let content = fs::read_to_string(path)?;
    if let Some(stripped) = content.strip_prefix("ref: ") {
        return resolve_ref_path(format!(".bit/{}", stripped.trim()));
    }
    Ok(content.trim().to_string())
}

// TODO: Maybe more efficient
fn list_ref_paths(path: impl AsRef<Path>) -> Result<Vec<String>, BitError> {
    let entries = fs::read_dir(path)?;
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();

        if file_type.is_dir() {
            files.extend(list_ref_paths(&path)?);
        } else {
            let path = path.to_str().expect("Non UTF8 path").to_string();
            files.push(path);
        }
    }

    Ok(files)
}
