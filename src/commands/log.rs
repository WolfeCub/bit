use clap::Args;
use colored::Colorize;

use crate::{
    commands::show_ref::resolve_ref,
    commit::Commit,
    object::{Object, ObjectType},
};

#[derive(Args, Debug)]
pub struct LogArg {
    pub commit: Option<String>,
}

impl LogArg {
    pub fn run(self) -> anyhow::Result<()> {
        let log_commit = self.commit.map_or_else(|| resolve_ref("HEAD"), Ok)?;
        for item in CommitIter::new(log_commit) {
            let (hash, commit) = item?;
            let (author, date) = commit.parse_author_date();

            // TODO: Extra ref info (HEAD -> main, tag, etc)
            println!("{} {}", "commit".yellow(), hash.yellow());
            println!("Author: {}", author);
            println!("Date:   {}", date.format("%a %h %d %H:%M:%S %Y %z"));
            println!();
            println!("    {}", commit.message);
        }

        Ok(())
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
