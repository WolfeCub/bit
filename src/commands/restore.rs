use std::fs;

use anyhow::Context;
use clap::Args;

use crate::{
    objects::{Ignore, Index, IndexEntry, Object, ObjectType, flatten_tree_from_disk},
    utils::{
        head::HeadState,
        path::{ArgListExpander, make_root_relative},
    },
};

/// Discards either working or staged changes for the given files
#[derive(Args, Debug)]
pub struct RestoreArg {
    pub files: Vec<String>,

    #[arg(short = 'W', long)]
    pub worktree: bool,

    #[arg(short = 'C', long)]
    pub staged: bool,
}

impl RestoreArg {
    fn worktree(&self) -> bool {
        self.worktree || (!self.worktree && !self.staged)
    }

    pub fn run(self) -> anyhow::Result<()> {
        let ignore = Ignore::build_from_disk()?;
        let expanded = ArgListExpander::new_recursive(&self.files, &ignore)?;

        let head_state = HeadState::read_from_disk()?;
        let head_commit = head_state.read_commit()?;

        let mut index = Index::parse_from_disk()?;
        for (path, _) in expanded {
            let normalized = make_root_relative(&path)?;
            let Some(index_entry_pos) = index.entries.iter().position(|ie| ie.name == normalized)
            else {
                continue;
            };

            if self.staged {
                let head_tree = head_commit
                    .as_ref()
                    .map(|commit| flatten_tree_from_disk(&commit.tree))
                    .context("could not resolve 'HEAD'. no commits")??;

                if let Some(old_tree_entry) = head_tree.get(&normalized) {
                    let hex = hex::decode(&old_tree_entry.hash)?
                        .try_into()
                        .map_err(|_| anyhow::anyhow!("Tree hash somehow invalid hex"))?;

                    // If the file exists in the head then we need to unstage it.
                    index.entries[index_entry_pos] = IndexEntry {
                        sha: hex,
                        flags: normalized.len().min(0xFFF) as u16,
                        name: normalized,
                        mode: u32::from_str_radix(&old_tree_entry.mode, 8)
                            .context("Mode was not octal")?,
                        ..Default::default()
                    }
                } else {
                    // If the file doesn't exist in the head then we can just remove it from the index
                    index.entries.remove(index_entry_pos);
                }
            }

            if self.worktree() {
                let hex = hex::encode(index.entries[index_entry_pos].sha);
                let object = Object::<Vec<u8>>::read_from_disk(&hex, ObjectType::Blob)?;

                fs::write(&path, &object.inner)?;
            }
        }

        if self.staged {
            index.write_to_disk()?;
        }

        Ok(())
    }
}
