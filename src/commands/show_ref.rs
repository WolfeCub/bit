use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;

use crate::{utils::repo_root};

#[derive(Args, Debug)]
pub struct ShowRefArg {}

impl ShowRefArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let mut refs = list_ref_paths(root.join(".bit/refs"))?;
        refs.sort();

        let prefix = root.join(".bit").canonicalize()?;
        let prefix = format!("{}/", prefix.to_str().expect("Non UTF8 path"));

        for r in refs.iter() {
            let hash = resolve_ref_path(r, &root)?;
            let path = r
                .strip_prefix(&prefix)
                .expect("Somehow ref doesn't start with $ROOT/.bit/");
            println!("{} {}", hash, path);
        }
        Ok(())
    }
}

pub fn resolve_ref(reference: &str) -> anyhow::Result<String> {
    let root = repo_root()?;
    let path = root.join(".bit").join(reference);

    resolve_ref_path(path, &root)
}

fn resolve_ref_path(path: impl AsRef<Path>, root: &PathBuf) -> anyhow::Result<String> {
    let content = fs::read_to_string(path)?;
    if let Some(stripped) = content.strip_prefix("ref: ") {
        let p = root.join(".bit").join(stripped.trim());
        return resolve_ref_path(p, root);
    }
    Ok(content.trim().to_string())
}

// TODO: Maybe more efficient
fn list_ref_paths(path: impl AsRef<Path>) -> anyhow::Result<Vec<String>> {
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
