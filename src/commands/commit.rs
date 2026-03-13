use std::fs;

use clap::Args;

use crate::{
    commands::{
        hash_object::hash_object_hex,
        write_tree::write_tree,
    },
    objects::{Commit, ObjectType},
    utils::{editor, get_user_info, git_time, head_state, repo_root},
};

#[derive(Args, Debug)]
pub struct CommitArg {
    #[arg(short, long)]
    pub message: Option<String>,
}

impl CommitArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;

        let head_state = head_state()?;

        // TODO: Support detached commits
        if head_state.detached {
            anyhow::bail!("Cannot commit in detached HEAD state");
        }

        let new_tree = write_tree(&root)?;

        let (user, email) = get_user_info();
        let author = format!("{} <{}> {}", user, email, git_time());

        let message = self.message.map_or_else(
            || {
                editor(
                    root.join(".bit/COMMIT_EDITMSG"),
                    &initial_commit_text(&head_state.name),
                )
            },
            Ok,
        )?;

        let commit = Commit {
            tree: new_tree,
            parent: Some(head_state.hash),
            author: author.clone(),
            committer: author,
            gpgsig: None, // TODO: Could be fun to support this
            message: message,
        };

        let commit_hash = hash_object_hex(ObjectType::Commit, commit, true)?;

        // Update HEAD to point to new commit, and update branch ref if HEAD is attached
        let path = root.join(".bit/refs/heads/").join(&head_state.name);
        fs::create_dir_all(path.parent().expect("Could not get parent directory"))?;
        fs::write(path, commit_hash)?;

        Ok(())
    }
}

fn initial_commit_text(branch_name: &str) -> String {
    format!(
        r"
# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# On branch {branch_name}
#
# Changes to be committed:
#   TODO: THIS IS HARDCODED
#	new file:   src/commands/commit.rs
#"
    )
}
