use clap::Args;
use itertools::Itertools;

use crate::{
    commands::{
        hash_object::hash_object_hex_from_disk, read_tree::read_tree, show_ref::resolve_ref,
    },
    objects::{Commit, Object, ObjectType, flatten_tree_from_disk},
    utils::{head::HeadState, repo::switch_head_to_branch},
};

#[derive(Args, Debug)]
pub struct SwitchArg {
    pub branch: String,
}

impl SwitchArg {
    pub fn run(self) -> anyhow::Result<()> {
        let branch_hash = resolve_ref(format!("refs/heads/{}", self.branch))?;
        let target_commit = Object::<Commit>::read_from_disk(&branch_hash, ObjectType::Commit)?;
        let target_flattened = flatten_tree_from_disk(&target_commit.inner.tree)?;

        let head_state = HeadState::read_from_disk()?;
        let head_hash = match head_state {
            HeadState::Unborn { .. } => None,
            HeadState::Attached { hash, .. } | HeadState::Detached { hash, .. } => Some(hash),
        };
        let head_flattened = head_hash
            .map(|hh| {
                let head_commit = Object::<Commit>::read_from_disk(&hh, ObjectType::Commit)?;
                flatten_tree_from_disk(&head_commit.inner.tree)
            })
            .transpose()?
            .unwrap_or_default();

        let mut conflicts = vec![];
        for (path, target_entry) in target_flattened.iter() {
            let object_hash = hash_object_hex_from_disk(&path, ObjectType::Blob, false)?;
            let branches_differ_and_file_changed = head_flattened
                .get(path)
                .map(|head_entry| {
                    head_entry.hash != target_entry.hash && head_entry.hash != object_hash
                })
                .unwrap_or(false);

            if branches_differ_and_file_changed {
                conflicts.push(path);
            }
        }
        // TODO: Print modified files when switching like so
        // M	foo
        // Switched to branch 'master'


        if conflicts.len() > 0 {
            anyhow::bail!(
                "Your local changes to the following files would be overwritten by checkout:\n{}\nPlease commit your changes or stash them before you switch branches.\nAborting",
                conflicts.iter().map(|c| format!("\t{}", c)).join("\n")
            );
        }

        read_tree(&target_commit.inner.tree, true)?;
        switch_head_to_branch(&self.branch)?;

        println!("Switched to branch '{}'", self.branch);

        Ok(())
    }
}
