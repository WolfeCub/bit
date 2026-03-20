use std::{fmt::Debug, fs};

use clap::Args;
use colored::Colorize;

use crate::{
    objects::{Ignore, Index, Object, ObjectType},
    utils::{
        changes::{UnstagedChangeKind, get_unstaged_changes},
        diff::{Edit, myers_diff},
        repo::repo_root,
    },
};

/// Shows changes between working directory, index, commits etc.
#[derive(Args, Debug)]
pub struct DiffArg {
    pub path: Option<String>,
}

impl DiffArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let index = Index::parse_from_disk()?;
        let ignore = Ignore::build_from_disk()?;

        let (unstaged_changes, _) = get_unstaged_changes(&index, &ignore)?;
        for c in unstaged_changes {
            if c.kind != UnstagedChangeKind::Modified {
                continue;
            }

            println!("{}", c.name);
            let old_content = Object::<String>::read_from_disk(&c.head_hash, ObjectType::Blob)?;
            let new_content = fs::read_to_string(root.join(c.name))?;

            let result = myers_diff(&old_content.inner, &new_content);
            // TODO: This might make more sense to be shared
            for (i, edit) in result.iter().enumerate() {
                let start = (i as isize - 3).max(0) as usize;
                let end = (i + 3).min(result.len() - 1);
                let near_change = result[start..=end]
                    .iter()
                    .any(|e| !matches!(e, Edit::Keep(_)));

                match edit {
                    Edit::Insert(line) => println!("{}", format!("+ {line}").green()),
                    Edit::Delete(line) => println!("{}", format!("- {line}").red()),
                    Edit::Keep(line) if near_change => println!("  {line}"),
                    _ => {}
                }
            }

            println!();
        }

        Ok(())
    }
}
