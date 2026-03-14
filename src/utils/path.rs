use std::path::{Component, Path, PathBuf};
use anyhow::Context;

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

pub fn relative_path(target: impl AsRef<Path>, base: impl AsRef<Path>) -> PathBuf {
    let mut target_components = target.as_ref().components().peekable();
    let mut base_components = base.as_ref().components().peekable();

    // Strip common prefix
    while target_components.peek() == base_components.peek() {
        target_components.next();
        base_components.next();
    }

    // For each remaining base component, add a ".."
    let mut result = PathBuf::new();
    for _ in base_components {
        result.push(Component::ParentDir);
    }
    for c in target_components {
        result.push(c);
    }
    result
}

pub fn relative_path_string(target: impl AsRef<Path>, base: impl AsRef<Path>) -> String {
    relative_path(target, base).to_string_lossy().to_string()
}


