use std::{collections::HashSet, fmt::Debug, fs};

use clap::Args;
use colored::Colorize;

use crate::{
    commands::hash_object::hash_object,
    objects::{Ignore, Index, Object, ObjectType},
    utils::{
        changes::{UnstagedChangeKind, get_changes_to_be_committed, get_unstaged_changes},
        diff::{Edit, compute_hunks, myers_diff},
        head::HeadState,
        pager,
        path::make_root_relative,
        repo::repo_root,
    },
};

/// Shows changes between working directory, index, commits etc.
#[derive(Args, Debug)]
pub struct DiffArg {
    pub paths: Vec<String>,

    #[arg(long)]
    pub cached: bool,
}

struct DiffChange {
    name: String,
    head_hash: Option<String>,
    new_hash: Option<String>,
    new_content: String,
}

impl DiffArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let index = Index::parse_from_disk()?;
        let ignore = Ignore::build_from_disk()?;

        let file_filters = HashSet::<String>::from_iter(
            self.paths
                .iter()
                .map(|p| make_root_relative(p))
                .collect::<anyhow::Result<Vec<_>>>()?,
        );

        // We always want to compare against the head commit as the "old" version.
        // If --cached is passed we want to compare the head commit to the index
        // i.e. we've added the change so it's in the index and how it differs from the last commit.
        // Otherwise we want to compare the head commit to the working directory
        // i.e. the content of the file hasn't made it to the index yet so just read it's contents.
        let changes = if self.cached {
            staged_changes(&index, &file_filters)?
        } else {
            unstaged_changes(root, index, ignore, file_filters)?
        };

        let mut output = vec![];
        for change in changes {
            let old_content = match &change.head_hash {
                Some(hash) => Object::<String>::read_from_disk(hash, ObjectType::Blob)?.inner,
                None => String::new(),
            };

            output.extend(render_file_diff(
                &change.name,
                change.head_hash.as_deref(),
                change.new_hash.as_deref(),
                &old_content,
                &change.new_content,
            )?);
        }

        pager(output.join("\n").as_str())?;

        Ok(())
    }
}

fn staged_changes(
    index: &Index,
    file_filters: &HashSet<String>,
) -> Result<Vec<DiffChange>, anyhow::Error> {
    let head_commit = HeadState::read_from_disk()?.read_commit()?;
    Ok(
        get_changes_to_be_committed(head_commit.map(|hc| hc.tree).as_deref(), index)?
            .into_iter()
            .filter(|c| file_filters.contains(c.name()) || file_filters.len() == 0)
            .map(|c| -> anyhow::Result<DiffChange> {
                let (new_hash, new_content) = match c.entry() {
                    Some(e) => {
                        let hash = hex::encode(e.sha);
                        let obj = Object::<String>::read_from_disk(&hash, ObjectType::Blob)?;
                        (Some(hash), obj.inner)
                    }
                    None => (None, String::new()),
                };

                Ok(DiffChange {
                    name: c.name().to_string(),
                    head_hash: c.head_hash().map(str::to_string),
                    new_hash,
                    new_content,
                })
            })
            .collect::<anyhow::Result<_>>()?,
    )
}

fn unstaged_changes(
    root: std::path::PathBuf,
    index: Index,
    ignore: Ignore,
    file_filters: HashSet<String>,
) -> Result<Vec<DiffChange>, anyhow::Error> {
    Ok(get_unstaged_changes(&index, &ignore)?
        .0
        .into_iter()
        .filter(|c| file_filters.contains(&c.name) || file_filters.len() == 0)
        .map(|c| -> anyhow::Result<DiffChange> {
            let (new_hash, new_content) = match c.kind {
                UnstagedChangeKind::Deleted => (None, String::new()),
                UnstagedChangeKind::Modified => {
                    let content = fs::read_to_string(root.join(&c.name))?;
                    let hash = hex::encode(hash_object(ObjectType::Blob, content.clone(), false)?);
                    (Some(hash), content)
                }
            };

            Ok(DiffChange {
                name: c.name,
                head_hash: Some(c.head_hash),
                new_hash,
                new_content,
            })
        })
        .collect::<anyhow::Result<_>>()?)
}

fn render_file_diff(
    name: &str,
    old_hash: Option<&str>,
    new_hash: Option<&str>,
    old_content: &str,
    new_content: &str,
) -> anyhow::Result<Vec<String>> {
    let result = myers_diff(old_content, new_content);
    let hunks = compute_hunks(&result);
    let mut output = vec![];
    output.push(format!("diff --git a/{0} b/{0}", name).bold().to_string());
    output.push(
        format!(
            "index {}..{} TODO MODE",
            old_hash.map(|h| &h[..7]).unwrap_or("0000000"),
            new_hash.map(|h| &h[..7]).unwrap_or("0000000"),
        )
        .bold()
        .to_string(),
    );

    let old_name = old_hash
        .map(|_| format!("a/{}", name))
        .unwrap_or_else(|| "/dev/null".to_string());
    let new_name = new_hash
        .map(|_| format!("b/{}", name))
        .unwrap_or_else(|| "/dev/null".to_string());
    output.push(format!("--- {}", old_name).bold().to_string());
    output.push(format!("+++ {}", new_name).bold().to_string());

    for hunk in hunks {
        output.push(
            format!(
                "@@ -{},{} +{},{} @@",
                hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
            )
            .cyan()
            .to_string(),
        );
        for edit in hunk.edits {
            output.push(match edit {
                Edit::Insert(line) => format!("+{line}").green().to_string(),
                Edit::Delete(line) => format!("-{line}").red().to_string(),
                Edit::Keep(line) => format!(" {line}"),
            });
        }
    }
    output.push(String::new());
    Ok(output)
}
