use std::{
    collections::HashMap,
    path::Path,
};

use clap::Args;

use crate::{
    commands::
        hash_object::{hash_object_hex, hash_object_hex_from_disk}
    ,
    objects::{Index, IndexEntry, ObjectType, Tree, TreeEntry},
    utils::repo::repo_root,
};

/// Writes the current index to a tree object and prints it's hash
#[derive(Args, Debug)]
pub struct WriteTreeArg {}

impl WriteTreeArg {
    pub fn run(self) -> anyhow::Result<()> {
        let hash = write_tree()?;
        println!("{}", hash);
        Ok(())
    }
}

/// Writes the current index to a tree object and returns its hash.
pub fn write_tree() -> anyhow::Result<String> {
    let index = Index::parse_from_disk()?;
    let tree = build_tree(&index.entries);

    write_tree_rec(&tree, &Path::new(""))
}

fn write_tree_rec(tree: &HashMap<String, TreeNode>, prefix: &Path) -> anyhow::Result<String> {
    let root = repo_root()?;

    let entries = tree
        .iter()
        .map(|(name, node)| {
            match node {
                TreeNode::File(entry) => {
                    let hash = hash_object_hex_from_disk(
                        root.join(&entry.name),
                        ObjectType::Blob,
                        true,
                    )?;

                    Ok(TreeEntry {
                        mode: format!("{:o}", entry.mode),
                        path: name.clone(),
                        hash,
                    })
                }

                TreeNode::Dir(subtree) => {
                    let hash = write_tree_rec(subtree, &prefix.join(name))?;

                    Ok(TreeEntry {
                        mode: "40000".to_string(),
                        path: name.clone(),
                        hash,
                    })
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    hash_object_hex(
        ObjectType::Tree,
        Tree { entries },
        true,
    )
}


#[derive(Debug)]
enum TreeNode {
    File(IndexEntry),
    Dir(HashMap<String, TreeNode>),
}

/// Turns the flat file path list from an index into a nested tree structure.
fn build_tree(entries: &[IndexEntry]) -> HashMap<String, TreeNode> {
    /// Recursive helper
    fn insert(
        mut map: HashMap<String, TreeNode>,
        parts: &[&str],
        entry: &IndexEntry,
    ) -> HashMap<String, TreeNode> {
        match parts {
            [file] => {
                map.insert(file.to_string(), TreeNode::File(entry.clone()));
            }
            [dir, rest @ ..] => {
                let subtree = match map.remove(*dir) {
                    Some(TreeNode::Dir(m)) => m,
                    _ => HashMap::new(),
                };
                map.insert(dir.to_string(), TreeNode::Dir(insert(subtree, rest, entry)));
            }
            [] => {
                unreachable!("Empty parts slice")
            }
        }
        map
    }

    // Split each entry's path into parts and then recurse down
    entries.iter().fold(HashMap::new(), |map, entry| {
        insert(
            map,
            entry.name.split('/').collect::<Vec<_>>().as_slice(),
            entry,
        )
    })
}
