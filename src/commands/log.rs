use std::collections::HashMap;

use clap::Args;
use colored::Colorize;
use itertools::Itertools;

use crate::objects::Commit;
use crate::utils::bit_dir_walker::BitDirWalker;
use crate::utils::repo::repo_root;
use crate::{
    commands::show_ref::resolve_ref,
    objects::{Ignore, Object, ObjectType},
};

#[derive(Args, Debug)]
pub struct LogArg {
    pub commit: Option<String>,
}

impl LogArg {
    pub fn run(self) -> anyhow::Result<()> {
        let map = build_hash_to_ref_map()?;

        let log_commit = self.commit.map_or_else(|| resolve_ref("HEAD"), Ok)?;
        for item in CommitIter::new(log_commit.clone()) {
            let (hash, commit) = item?;
            let (author, date) = commit.parse_author_date();

            let hash_names = if let Some(names) = map.get(&hash) {
                let joined = names
                    .iter()
                    .map(|f| format_ref(f, hash == log_commit))
                    .join(", ".yellow().as_ref());

                format!("{}{}{}", " (".yellow(), joined, ")".yellow())
            } else {
                String::new()
            };

            println!("{} {}{}", "commit".yellow(), hash.yellow(), hash_names);
            println!("Author: {}", author);
            println!("Date:   {}", date.format("%a %h %d %H:%M:%S %Y %z"));
            println!();
            println!("    {}", commit.message);
        }

        Ok(())
    }
}

fn build_hash_to_ref_map() -> anyhow::Result<HashMap<String, Vec<String>>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let ignore = Ignore::build_from_disk()?;
    let root = repo_root()?;
    let refs_dir = root.join(".bit/refs");

    for entry in BitDirWalker::new(&refs_dir, ignore)? {
        if entry.file_type()?.is_file() {
            let ref_name = entry
                .path()
                .strip_prefix(&refs_dir)?
                .to_string_lossy()
                .into_owned();
            let hash = resolve_ref(refs_dir.join(&ref_name))?;
            map.entry(hash).or_default().push(ref_name);
        }
    }
    Ok(map)
}

fn format_ref(ref_name: &str, is_head: bool) -> String {
    let formatted = if let Some(branch) = ref_name.strip_prefix("heads/") {
        branch.green().to_string()
    } else if let Some(remote) = ref_name.strip_prefix("remotes/") {
        remote.red().to_string()
    } else if let Some(tag) = ref_name.strip_prefix("tags/") {
        format!("tag: {}", tag).yellow().to_string()
    } else {
        ref_name.yellow().to_string()
    };

    if is_head {
        format!("{} {} {}", "HEAD".cyan(), "->".yellow(), formatted)
    } else {
        formatted
    }
}

struct CommitIter {
    next_commit: Option<String>,
}

impl CommitIter {
    pub fn new(hash: String) -> Self {
        Self {
            next_commit: Some(hash),
        }
    }
}

impl Iterator for CommitIter {
    type Item = anyhow::Result<(String, Commit)>;

    fn next(&mut self) -> Option<Self::Item> {
        let hash = self.next_commit.as_ref()?.clone();
        let object = Object::<Commit>::read_from_disk(&hash, ObjectType::Commit);

        match object.map(|o| o.inner) {
            Ok(commit) => {
                self.next_commit = commit.parent.clone();

                Some(Ok((hash, commit)))
            }
            Err(e) => {
                self.next_commit = None;
                Some(Err(e))
            }
        }
    }
}
