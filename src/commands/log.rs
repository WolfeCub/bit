use clap::Args;

use crate::{
    commit::Commit,
    errors::BitError,
    object::{Object, ObjectType},
};

#[derive(Args, Debug)]
pub struct LogArg {
    pub commit: Option<String>,
}

impl LogArg {
    pub fn run(self) -> Result<(), BitError> {
        for c in CommitIter::new(self.commit.unwrap()) {
            let commit = c?;
            let (author, date) = commit.parse_author_date();

            // TODO: Color
            println!("commit {}", commit.hash);
            println!("Author: {}", author);
            println!("Date:   {}", date.format("%a %h %d %H:%M:%S %Y %z"));
            println!();
            println!("{}", commit.message);
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
    type Item = Result<Commit, BitError>;

    fn next(&mut self) -> Option<Self::Item> {
        let hash = self.next_commit.as_ref()?;
        let object = Object::read_from_disk(ObjectType::Commit, hash);
        let commit = Commit::parse(hash.clone(), &object.content);

        let Ok(commit) = commit else {
            self.next_commit = None;
            return Some(commit);
        };

        self.next_commit = commit.parent.clone();

        Some(Ok(commit))
    }
}
