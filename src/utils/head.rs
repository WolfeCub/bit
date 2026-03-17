use std::{fs, io};

use crate::{commands::show_ref::resolve_ref, utils::repo::repo_root};

#[derive(Debug)]
pub enum HeadState {
    Unborn {
        branch: String,
    },
    Attached {
        branch: String,
        hash: String,
    },
    Detached {
        hash: String,
        /// TODO: e.g. tag name
        #[allow(dead_code, unused)]
        label: Option<String>,
    },
}

impl HeadState {
    pub fn read_from_disk() -> anyhow::Result<Self> {
        let root = repo_root()?;
        let head = fs::read_to_string(root.join(".bit/HEAD"))?;
        let head = head.trim();

        if let Some(refs_path) = head.strip_prefix("ref: ") {
            let branch = refs_path
                .strip_prefix("refs/heads/")
                .unwrap_or(refs_path)
                .to_string();

            match resolve_ref(refs_path) {
                Ok(hash) => Ok(HeadState::Attached { branch, hash }),
                // We assume we have no commits if we have a validly formatted ref but it doesn't exist on disk.
                Err(e)
                    if e.downcast_ref::<io::Error>()
                        .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
                {
                    Ok(HeadState::Unborn { branch })
                }
                Err(e) => Err(dbg!(e)),
            }
        } else {
            Ok(HeadState::Detached {
                hash: head.to_string(),
                label: None,
            }) // TODO: resolve tag
        }
    }

    pub fn branch_name(&self) -> Option<&str> {
        match self {
            HeadState::Unborn { branch } | HeadState::Attached { branch, .. } => Some(branch),
            HeadState::Detached { .. } => None,
        }
    }
}
