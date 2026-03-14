use std::{fs, path::PathBuf};

use clap::Args;

use crate::{
    commands::{hash_object::hash_object_hex, show_ref::resolve_ref},
    objects::{ObjectType, Tag},
    utils::{config::get_user_info, editor, git_time, repo::repo_root},
};

/// Creates a new tag or lists existing tags if no name is provided
#[derive(Args, Debug)]
pub struct TagArg {
    /// Create a tag object instead of a lightweight tag. This now also requires a message to be provided
    #[arg(short = 'a', default_value_t = false, requires = "name")]
    pub tag_object: bool,

    #[arg(short, long, requires = "tag_object")]
    pub message: Option<String>,

    pub name: Option<String>,
    pub object: Option<String>,
}

impl TagArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;

        let Some(tag_name) = self.name else {
            for tag in list_tags(&root)? {
                println!("{}", tag);
            }
            return Ok(());
        };

        let object = match self.object {
            Some(object) => object,
            None => resolve_ref("HEAD")?,
        };

        let hash_to_write = if self.tag_object {
            let (user, email) = get_user_info();
            let author = format!("{} <{}> {}", user, email, git_time());

            let message = self.message.map_or_else(
                || editor(root.join(".bit/TAG_EDITMSG"), &initial_tag_text(&tag_name)),
                Ok,
            )?;

            let tag = Tag {
                object,
                type_: ObjectType::Commit,
                tag: tag_name.clone(),
                tagger: author,
                message,
            };
            hash_object_hex(ObjectType::Tag, tag, true)?
        } else {
            object
        };

        let path = root.join(".bit/refs/tags").join(&tag_name);
        fs::create_dir_all(path.parent().expect("Could not get parent directory"))?;
        fs::write(path, hash_to_write)?;

        Ok(())
    }
}

fn list_tags(root: &PathBuf) -> anyhow::Result<Vec<String>> {
    let files = fs::read_dir(root.join(".bit/refs/tags"))?;
    files
        .map(|f| {
            Ok(f?
                .file_name()
                .to_str()
                .expect("Non UTF8 tag name")
                .to_string())
        })
        .collect()
}

fn initial_tag_text(tag_name: &str) -> String {
    format!(
        r"
#
# Write a message for tag:
#   {}
# Lines starting with '#' will be ignored.",
        tag_name
    )
}
