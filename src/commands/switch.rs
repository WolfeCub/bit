use std::collections::HashSet;

use clap::Args;
use itertools::Itertools;

use crate::{
    commands::{
        hash_object::hash_object_hex_from_disk, read_tree::read_flattened_tree_ignorelist,
        show_ref::resolve_ref,
    },
    objects::{Commit, Object, ObjectType, flatten_tree_from_disk},
    utils::{head::HeadState, repo::switch_head_to_branch},
};

/// Switch between branches
#[derive(Args, Debug)]
pub struct SwitchArg {
    pub branch: String,

    #[arg(short, long)]
    pub create: bool,
}

impl SwitchArg {
    pub fn run(self) -> anyhow::Result<()> {
        let head_state = HeadState::read_from_disk()?;

        // If we create a new branch it's tree is exactly our tree. So there won't be any conflicts.
        if !self.create {
            let head_flattened = head_state
                .read_commit()?
                .map(|commit| flatten_tree_from_disk(&commit.tree))
                .transpose()?
                .unwrap_or_default();

            let branch_hash = resolve_ref(format!("refs/heads/{}", self.branch))?;
            let target_commit = Object::<Commit>::read_from_disk(&branch_hash, ObjectType::Commit)?;
            let target_flattened = flatten_tree_from_disk(&target_commit.inner.tree)?;

            let mut conflicts = vec![];
            let mut changes = HashSet::new();
            for (path, target_entry) in target_flattened.iter() {
                let object_hash = hash_object_hex_from_disk(&path, ObjectType::Blob, false)?;
                let (branches_differ, file_changed) = head_flattened
                    .get(path)
                    .map(|head_entry| {
                        (
                            head_entry.hash != target_entry.hash,
                            head_entry.hash != object_hash,
                        )
                    })
                    .unwrap_or((false, false));

                if branches_differ && file_changed {
                    conflicts.push(path);
                }

                if file_changed {
                    changes.insert(path.clone());
                }
            }

            if conflicts.len() > 0 {
                anyhow::bail!(
                    "Your local changes to the following files would be overwritten by checkout:\n{}\nPlease commit your changes or stash them before you switch branches.\nAborting",
                    conflicts.iter().map(|c| format!("\t{}", c)).join("\n")
                );
            }

            if changes.len() > 0 {
                println!("{}", changes.iter().map(|c| format!("M\t{}", c)).join("\n"));
            }

            read_flattened_tree_ignorelist(target_flattened, true, Some(changes))?;
        }

        switch_head_to_branch(&self.branch)?;

        println!("Switched to branch '{}'", self.branch);

        Ok(())
    }
}
