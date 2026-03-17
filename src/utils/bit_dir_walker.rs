use std::fs::{self, DirEntry, ReadDir};
use std::path::{Path, PathBuf};

use crate::objects::Ignore;

/// An iterator that recursively walks the dir tree yields all files (no dirs) and respects .bitignore rules.
pub struct BitDirWalker<'a> {
    root: PathBuf,
    stack: Vec<ReadDir>,
    ignore: Option<&'a Ignore>,
}

impl<'a> BitDirWalker<'a> {
    pub fn new(root: impl AsRef<Path>) -> std::io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        let stack = vec![fs::read_dir(&root)?];
        Ok(Self {
            root,
            stack,
            ignore: None,
        })
    }
    pub fn new_with_ignore(root: impl AsRef<Path>, ignore: &'a Ignore) -> std::io::Result<Self> {
        Ok(Self {
            ignore: Some(ignore),
            ..BitDirWalker::new(root)?
        })
    }
}

impl<'a> Iterator for BitDirWalker<'a> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let top = self.stack.last_mut()?;

            match top.next() {
                None => {
                    self.stack.pop();
                }
                Some(Err(_)) => continue,
                Some(Ok(entry)) => {
                    let path = entry.path();
                    let Ok(rel) = path.strip_prefix(&self.root) else {
                        continue;
                    };
                    let Some(rel) = rel.to_str() else { continue };
                    let is_dir = entry.file_type().is_ok_and(|ft| ft.is_dir());

                    if self
                        .ignore
                        .as_ref()
                        .map(|i| i.is_file_ignored(rel, is_dir))
                        .unwrap_or(false)
                    {
                        continue;
                    }

                    if is_dir {
                        if let Ok(rd) = fs::read_dir(entry.path()) {
                            self.stack.push(rd);
                        }
                    } else {
                        return Some(entry);
                    }
                }
            }
        }
    }
}
