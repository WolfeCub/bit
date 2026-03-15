use std::{env, fs, path::PathBuf};

use anyhow::Context;
use cached::proc_macro::cached;

use crate::commands::show_ref::resolve_ref;

#[cached(result = true)]
pub fn repo_root() -> anyhow::Result<PathBuf> {
    let mut cwd = cwd()?;
    loop {
        if cwd.join(".bit").exists() {
            return Ok(cwd);
        }

        if let Some(next) = cwd.parent() {
            cwd = next.to_path_buf();
        } else {
            anyhow::bail!("Not a bit repository (or any of the parent directories)");
        }
    }
}


/// Thin wrapper around `current_dir` that provides caching so we can reuse
/// The working directly won't change during any git/bit calls
#[cached(result = true)]
pub fn cwd() -> anyhow::Result<PathBuf> {
    Ok(env::current_dir()?)
}

pub fn object_path(hash: &str) -> anyhow::Result<PathBuf> {
    Ok(repo_root()?
        .join(".bit/objects")
        .join(&hash[..2])
        .join(&hash[2..]))
}

pub fn find_hash(target: &str) -> anyhow::Result<String> {
    // Min length for a shortened hash is 4
    if target.len() >= 4 {
        let dir = repo_root()?.join(".bit/objects").join(&target[..2]);
        let entries = fs::read_dir(dir)
            .map(|rd| rd.collect::<Vec<_>>())
            .unwrap_or_default();

        // If there's exactly one hash and it's prefix matches target return it
        if let [Ok(e)] = entries.as_slice() {
            let file_name = e.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name.starts_with(&target[2..]) {
                // Reconstruct the full hash
                return Ok(format!("{}{}", &target[..2], file_name));
            }
        }
    }

    // Priority list of search paths
    // i.e. refs/tags/name works but so does tags/name and just name
    // "" is repo root this covers things like: HEAD, ORIG_HEAD, MERGE_HEAD, etc
    // TODO: Currently the root is too loosey goosey `rev-parse COMMIT_EDITMSG` will just dump the file
    ["", "refs/", "refs/tags/", "refs/heads/", "refs/remotes/"]
        .iter()
        .find_map(|prefix| resolve_ref(&format!("{}{}", prefix, target)).ok())
        .context("Unable to resolve or ambiguous hash or ref")
}
