use std::{
    fs::File,
    io::Read,
    os::unix::fs::MetadataExt,
};

use anyhow::Context;
use clap::Args;

use crate::{
    commands::{hash_object::hash_object, remove},
    objects::{IndexEntry, ObjectType, TimePair}, utils::make_root_relative,
};

#[derive(Args, Debug)]
pub struct AddArg {
    pub paths: Vec<String>,
}

impl AddArg {
    pub fn run(self) -> anyhow::Result<()> {
        let mut new_index = remove::remove(&self.paths, false)?;

        // TODO: Remove already does this. We're duplicating work here.
        // we can maybe annotate it with #[cached] or pass around the normalized_paths
        let normalized_paths = self.paths.iter().map(make_root_relative).collect::<anyhow::Result<Vec<_>>>()?;

        for (path, normalized) in self.paths.into_iter().zip(normalized_paths.into_iter()) {
            let mut file =
                File::open(&path).with_context(|| format!("Failed to open file '{}'", &path))?;

            let metadata = file
                .metadata()
                .with_context(|| format!("Failed to read metadata for '{}'", path))?;

            let mut bytes = Vec::with_capacity(metadata.len() as usize);
            file.read_to_end(&mut bytes)
                .with_context(|| format!("Failed to read file '{}'", path))?;

            let hash = hash_object(ObjectType::Blob, bytes, true)?;

            // TODO: This is linux only currently
            let entry = IndexEntry {
                ctime: TimePair {
                    s: u32::try_from(metadata.ctime())?,
                    ns: u32::try_from(metadata.ctime_nsec())?,
                },
                mtime: TimePair {
                    s: u32::try_from(metadata.mtime())?,
                    ns: u32::try_from(metadata.mtime_nsec())?,
                },
                dev: u32::try_from(metadata.dev())?,
                ino: u32::try_from(metadata.ino())?,
                mode: metadata.mode(),
                uid: metadata.uid(),
                gid: metadata.gid(),
                size: u32::try_from(metadata.size())?,
                sha: hash,
                flags: normalized.len().min(0xFFF) as u16, // TODO: We're ignoring the upper set bits for now
                name: normalized,
            };

            new_index.entries.push(entry);
        }

        new_index.entries.sort_by(|a, b| a.name.cmp(&b.name));

        new_index.write_to_disk()?;

        Ok(())
    }
}
