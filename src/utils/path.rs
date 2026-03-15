use anyhow::Context;
use pathdiff::diff_paths;
use std::
    path::Path
;

use crate::utils::repo::repo_root;

/// Converts a path to be relative to the repo root. Paths must be within the repo.
pub fn make_root_relative(path: impl AsRef<Path>) -> anyhow::Result<String> {
    let root = repo_root()?;

    let absolute_path = path.as_ref().canonicalize()?;
    let repo_relative_path = absolute_path
        .strip_prefix(&root)
        .with_context(|| format!("Path {absolute_path:?} is not within the repository"))?;

    Ok(repo_relative_path.to_string_lossy().into())
}

pub fn relative_path_string(
    target: impl AsRef<Path>,
    base: impl AsRef<Path>,
) -> anyhow::Result<String> {
    Ok(diff_paths(target, base)
        .context("Unable to compute relative path")?
        .to_string_lossy()
        .to_string())
}
