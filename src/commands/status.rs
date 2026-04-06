use clap::Args;
use colored::Colorize;

use crate::{
    objects::{
        Commit, Ignore, Index, Object,
        ObjectType::{self},
    },
    utils::{
        changes::{
            StagedChange, UnstagedChange, UnstagedChangeKind, get_changes_to_be_committed,
            get_unstaged_changes,
        },
        head::HeadState,
        path::relative_path_string,
        repo::{cwd, repo_root},
    },
};

/// Shows the current branch, staged changes, unstaged changes and untracked files
#[derive(Args, Debug)]
pub struct StatusArg {}

// TODO: Supporting renames would be cool
impl StatusArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let head_state = HeadState::read_from_disk()?;

        let head_hash = match head_state {
            HeadState::Detached { hash, .. } => {
                println!("HEAD detached at {}", hash);
                Some(hash)
            }
            HeadState::Attached { branch, hash } => {
                println!("On branch {}", branch);
                Some(hash)
            }
            HeadState::Unborn { branch } => {
                println!("On branch {}\n\nNo commits yet", branch);
                None
            }
        };

        let ignore = Ignore::build_from_disk()?;
        let index = Index::parse_from_disk()?;

        let head_commit = head_hash
            .map(|hh| -> anyhow::Result<String> {
                Ok(Object::<Commit>::read_from_disk(&hh, ObjectType::Commit)?
                    .inner
                    .tree)
            })
            .transpose()?;
        let changes = get_changes_to_be_committed_text(head_commit.as_deref(), &index)?;
        if changes.len() > 0 {
            println!("\nChanges to be committed:");
            for change in changes {
                println!("{}", change.green());
            }
        }

        let (unstaged_changes, untracked_files) = get_unstaged_changes(&index, &ignore)?;

        if unstaged_changes.len() > 0 {
            println!(
                "\nChanges not staged for commit:\n{}",
                format_unstaged_changes(&unstaged_changes)?
            );
        }

        if untracked_files.len() > 0 {
            println!("\nUntracked files:");
        }
        // Anything that's left over in the files list is untracked
        for path in untracked_files {
            let relative = relative_path_string(&root.join(&path), cwd()?)?;
            println!("        {}", relative.red());
        }

        Ok(())
    }
}

pub fn get_changes_to_be_committed_text(
    tree_hash: Option<&str>,
    index: &Index,
) -> anyhow::Result<Vec<String>> {
    let root = repo_root()?;
    let cwd = cwd()?;

    let changes = get_changes_to_be_committed(tree_hash, index)?;
    changes
        .into_iter()
        .map(|change| {
            let relative = relative_path_string(&root.join(&change.name()), &cwd)?;

            let result = match change {
                StagedChange::Added(_) => format!("        new file:   {}", &relative),
                StagedChange::Modified { .. } => format!("        modified:   {}", &relative),
                StagedChange::Deleted { .. } => format!("        deleted:    {}", &relative),
            };

            Ok(result)
        })
        .collect()
}

fn format_unstaged_changes(changes: &[UnstagedChange]) -> anyhow::Result<String> {
    let root = repo_root()?;
    let cwd = cwd()?;

    let formatted = changes
        .into_iter()
        .map(|change| {
            let relative = relative_path_string(&root.join(&change.name), &cwd)?;
            let text = match change.kind {
                UnstagedChangeKind::Deleted => format!("        deleted:    {}", relative),
                UnstagedChangeKind::Modified => format!("        modified:   {}", relative),
            };
            Ok(text.red().to_string())
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .join("\n");

    Ok(formatted)
}
