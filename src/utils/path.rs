use anyhow::Context;
use pathdiff::diff_paths;
use std::{
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

use crate::{
    objects::Ignore,
    utils::{bit_dir_walker::BitDirWalker, repo::repo_root},
};

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

#[derive(Debug, Clone)]
pub struct ArgListExpander<'a> {
    worklist: Vec<(PathBuf, Metadata)>,
    ignore: &'a Ignore,
}

impl<'a> ArgListExpander<'a> {
    pub fn new_recursive(args: &[String], ignore: &'a Ignore) -> anyhow::Result<Self> {
        Self::new(args, ignore, true)
    }

    pub fn new(args: &[String], ignore: &'a Ignore, recursive: bool) -> anyhow::Result<Self> {
        let worklist = args
            .iter()
            .map(|p| -> anyhow::Result<(PathBuf, Metadata)> {
                let meta = fs::metadata(&p)?;
                if !recursive && meta.is_dir() {
                    anyhow::bail!("not removing '{}' recursively without -r", &p);
                }
                Ok((p.into(), meta))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self { worklist, ignore })
    }
}

impl<'a> Iterator for ArgListExpander<'a> {
    type Item = (PathBuf, Metadata);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((path, metadata)) = self.worklist.pop() {
            if metadata.is_file() {
                return Some((path, metadata));
            } else {
                let work = BitDirWalker::new_with_ignore(&path, self.ignore)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.metadata().ok().map(|meta| (e.path(), meta)));

                self.worklist.extend(work);
            }
        }

        None
    }
}
