use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use anyhow::Context;
use clap::Args;

use crate::utils::{
    bit_dir_walker::BitDirWalker,
    head::HeadState,
    repo::{find_hash, repo_root, switch_head_to_branch},
};
use colored::Colorize;

// TODO: Maybe a custom argument verifier here
/// Creates, lists, deletes, or moves branches
#[derive(Args, Debug)]
pub struct BranchArg {
    pub branch: Option<String>,

    #[arg(conflicts_with = "delete")]
    pub second_arg: Option<String>,

    #[arg(long, requires = "branch")]
    pub force: bool,

    // TODO: -M and -D are aliases of -m and -d with --force
    #[arg(short, long, requires = "branch")]
    pub delete: bool,

    #[arg(short, long, requires = "branch")]
    pub move_: bool,
}

impl BranchArg {
    pub fn run(self) -> anyhow::Result<()> {
        let head_state = HeadState::read_from_disk()?;

        if self.branch.is_none() {
            return print_branches(&head_state);
        }

        let Some(branch) = self.branch else {
            anyhow::bail!("Branch name is required");
        };

        if self.move_ {
            let (old_branch, new_branch) = if let Some(second_arg) = self.second_arg {
                (branch, second_arg)
            } else {
                let Some(branch_name) = head_state.branch_name() else {
                    anyhow::bail!("cannot rename the current branch while not on any");
                };
                (branch_name.to_owned(), branch)
            };

            move_branch(&old_branch, &new_branch, self.force)?;

            if head_state.branch_name().is_some_and(|b| b == old_branch) {
                switch_head_to_branch(&new_branch)?;
            }
            return Ok(());
        }

        let start_point = self.second_arg.unwrap_or_else(|| "HEAD".to_string());
        let start_hash = find_hash(&start_point)?;

        if self.delete {
            if head_state.branch_name().is_some_and(|b| b == branch) {
                anyhow::bail!("Cannot delete the current branch");
            }
            let deleted_hash = delete_branch(&branch, self.force)?;
            println!("Deleted branch {} (was {})", &branch, &deleted_hash[..7]);
            return Ok(());
        }

        create_branch(&branch, &start_hash, self.force)?;

        Ok(())
    }
}

pub fn create_branch(branch: &str, start_hash: &str, force: bool) -> anyhow::Result<()> {
    let root = repo_root()?;
    let path = root.join(".bit/refs/heads").join(branch);
    OpenOptions::new()
        .create(true)
        .create_new(!force)
        .write(true)
        .open(path)
        .map_err(|e| {
            // This error can only happen when create_new is true
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                anyhow::anyhow!("Branch '{}' already exists", branch)
            } else {
                e.into()
            }
        })?
        .write_all(start_hash.as_bytes())?;

    Ok(())
}

fn move_branch(old_branch: &str, new_branch: &str, _force: bool) -> anyhow::Result<()> {
    let root = repo_root()?;
    let heads = root.join(".bit/refs/heads");
    let new_path = heads.join(new_branch);
    fs::create_dir_all(
        new_path
            .parent()
            .context("Unable to get parent directory")?,
    )?;
    fs::rename(heads.join(old_branch), new_path)?;
    Ok(())
}

// TODO: Delete should check if the branch is merged and refuse to delete if it isn't
fn delete_branch(branch: &str, _force: bool) -> anyhow::Result<String> {
    let root = repo_root()?;
    let path = root.join(".bit/refs/heads").join(branch);
    let old_hash = fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("Branch '{}' not found", branch)
        } else {
            e.into()
        }
    })?;
    fs::remove_file(path)?;
    Ok(old_hash)
}

pub fn get_branches() -> anyhow::Result<Vec<String>> {
    let root = repo_root()?;
    let heads_dir = root.join(".bit/refs/heads");
    let walker = BitDirWalker::new(&heads_dir)?;

    let mut branches = walker
        .map(|e| -> anyhow::Result<String> {
            Ok(e.path()
                .canonicalize()?
                .strip_prefix(&heads_dir)?
                .to_string_lossy()
                .to_string())
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    branches.sort();

    Ok(branches)
}

fn print_branches(head_state: &HeadState) -> anyhow::Result<()> {
    let branches = get_branches()?;

    let branch = match head_state {
        HeadState::Unborn { branch } | HeadState::Attached { branch, .. } => Some(branch),
        HeadState::Detached { hash, .. } => {
            println!("* {}", format!("(HEAD detached at {})", &hash[..7]).green());
            None
        }
    };

    for b in branches {
        if branch.is_some_and(|br| *br == b) {
            println!("* {}", b.green());
        } else {
            println!("  {}", b);
        }
    }

    Ok(())
}
