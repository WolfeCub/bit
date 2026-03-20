use std::{fmt::Debug, fs};

use clap::Args;
use colored::Colorize;

use crate::{
    commands::hash_object::hash_object,
    objects::{Ignore, Index, Object, ObjectType},
    utils::{
        changes::{UnstagedChangeKind, get_unstaged_changes},
        diff::{Edit, compute_hunks, myers_diff},
        repo::repo_root,
    },
};

/// Shows changes between working directory, index, commits etc.
#[derive(Args, Debug)]
pub struct DiffArg {
    pub paths: Vec<String>,
}

impl DiffArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let index = Index::parse_from_disk()?;
        let ignore = Ignore::build_from_disk()?;

        let (unstaged_changes, _) = get_unstaged_changes(&index, &ignore)?;
        for change in unstaged_changes {
            if change.kind != UnstagedChangeKind::Modified {
                continue;
            }

            let old_content =
                Object::<String>::read_from_disk(&change.head_hash, ObjectType::Blob)?;
            let new_content = fs::read_to_string(root.join(&change.name))?;
            let new_hash = hash_object(ObjectType::Blob, new_content.clone(), false)?;

            let result = myers_diff(&old_content.inner, &new_content);
            let hunks = compute_hunks(&result);

            println!(
                "{}\n{}\n{}\n{}",
                format!("diff --git a/{0} b/{0}", &change.name).bold(),
                format!(
                    "index {}..{} {}",
                    &change.head_hash[..7],
                    &hex::encode(new_hash)[..7],
                    "TODO MODE"
                )
                .bold(),
                format!("--- a/{}", &change.name).bold(),
                format!("+++ b/{}", &change.name).bold(),
            );

            for hunk in hunks {
                println!(
                    "{}",
                    format!(
                        "@@ -{},{} +{},{} @@",
                        hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count,
                    )
                    .cyan()
                );

                for edit in hunk.edits {
                    match edit {
                        Edit::Insert(line) => println!("{}", format!("+ {line}").green()),
                        Edit::Delete(line) => println!("{}", format!("- {line}").red()),
                        Edit::Keep(line) => println!("  {line}"),
                    }
                }
            }

            println!();
        }

        Ok(())
    }
}
