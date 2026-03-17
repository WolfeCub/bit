use std::fs;

use clap::Args;

use crate::{
    commands::{
        hash_object::hash_object_hex, status::get_changes_to_be_committed_text,
        write_tree::write_tree,
    },
    objects::{Commit, Index, ObjectType},
    utils::{config::get_user_info, editor, git_time, head::HeadState, repo::repo_root},
};

/// Creates a new commit with the current index as the tree, and HEAD as the parent
#[derive(Args, Debug)]
pub struct CommitArg {
    #[arg(short, long)]
    pub message: Option<String>,
}

impl CommitArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;

        let head_state = HeadState::read_from_disk()?;

        let (head_hash, head_branch) = match head_state {
            // TODO: Support detached commits
            HeadState::Detached { .. } => {
                anyhow::bail!("Cannot commit in detached HEAD state");
            }
            HeadState::Attached { hash, branch } => (Some(hash), branch),
            HeadState::Unborn { branch } => (None, branch),
        };

        let new_tree = write_tree()?;

        let (user, email) = get_user_info();
        let author = format!("{} <{}> {}", user, email, git_time());

        let message = self
            .message
            .map_or_else(|| open_commit_editor(head_hash.as_deref(), &head_branch), Ok)?;

        let commit = Commit {
            tree: new_tree,
            parent: head_hash,
            author: author.clone(),
            committer: author,
            gpgsig: None, // TODO: Could be fun to support this
            message: message,
        };

        let commit_hash = hash_object_hex(ObjectType::Commit, commit, true)?;

        // Update HEAD to point to new commit, and update branch ref if HEAD is attached
        let path = root.join(".bit/refs/heads/").join(&head_branch);
        fs::create_dir_all(path.parent().expect("Could not get parent directory"))?;
        fs::write(path, commit_hash)?;

        Ok(())
    }
}

fn open_commit_editor(
    head_hash: Option<&str>,
    head_branch: &String,
) -> Result<String, anyhow::Error> {
    let root = repo_root()?;
    let index = Index::parse_from_disk()?;

    let changes = get_changes_to_be_committed_text(head_hash, &index)?;
    editor(
        root.join(".bit/COMMIT_EDITMSG"),
        &initial_commit_text(head_branch, changes),
    )
}

fn initial_commit_text(branch_name: &str, changes: Vec<String>) -> String {
    let change_lines = changes.join("\n");

    format!(
        r"
# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# On branch {branch_name}
#
# Changes to be committed:
#{change_lines}
#"
    )
}
