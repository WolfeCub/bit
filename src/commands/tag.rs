use std::{fs, path::PathBuf};

use clap::Args;

use crate::{
    commands::{hash_object::hash_object, show_ref::resolve_ref},
    errors::BitError,
    object::ObjectType,
    tag::Tag,
    util::{editor, git_time, repo_root},
};

#[derive(Args, Debug)]
pub struct TagArg {
    #[arg(short = 'a', default_value_t = false)]
    pub tag_object: bool,

    #[arg(required_if_eq("tag_object", "true"))]
    pub name: Option<String>,
    pub object: Option<String>,
}

impl TagArg {
    pub fn run(self) -> Result<(), BitError> {
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
            // TODO: When we have a config file
            let author = format!("{} <{}> {}", "Todo Name", "todo@email.ca", git_time());

            let message = editor(root.join(".bit/TAG_EDITMSG"), &initial_tag_text(&tag_name))?;

            let tag = Tag {
                object,
                type_: ObjectType::Commit,
                tag: tag_name.clone(),
                tagger: author,
                message,
            };
            hash_object(ObjectType::Tag, tag, true)?
        } else {
            object
        };

        let path = root.join(".bit/refs/tags").join(&tag_name);
        fs::create_dir_all(path.parent().expect("Could not get parent directory"))?;
        fs::write(path, hash_to_write)?;

        Ok(())
    }
}

fn list_tags(root: &PathBuf) -> Result<Vec<String>, BitError> {
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
