use clap::Args;

use crate::utils::{bit_dir_walker::BitDirWalker, head::HeadState, repo::repo_root};
use colored::Colorize;

/// Creates, lists, deletes, or moves branches
#[derive(Args, Debug)]
pub struct BranchArg {
    pub branch: Option<String>,
}

impl BranchArg {
    pub fn run(self) -> anyhow::Result<()> {
        if self.branch.is_none() {
            return print_branches();
        }

        Ok(())
    }
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

fn print_branches() -> anyhow::Result<()> {
    let branches = get_branches()?;
    let head_state = HeadState::read_from_disk()?;

    let branch = match head_state {
        HeadState::Unborn { branch } | HeadState::Attached { branch, .. } => Some(branch),
        HeadState::Detached { hash, .. } => {
            println!("* {}", format!("(HEAD detached at {})", &hash[..7]).green());
            None
        }
    };

    for b in branches {
        if branch.as_ref().is_some_and(|br| *br == b) {
            println!("* {}", b.green());
        } else {
            println!("  {}", b);
        }
    }

    Ok(())
}
