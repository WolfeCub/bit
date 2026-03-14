use anyhow::Context;
use std::{
    fs,
    io::{self, BufReader, Read, Seek},
    mem,
    path::Path,
};

use crate::utils::repo::repo_root;

#[derive(Debug)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
}

// TODO: We currently don't support extensions which are technically valid in version 2
impl Index {
    pub fn from_entries(entries: Vec<IndexEntry>) -> Self {
        Self { entries }
    }

    pub fn parse_from_disk() -> anyhow::Result<Self> {
        let path = repo_root()?.join(".bit/index");

        let mut reader = IndexReader::new(&path)?;

        if reader.read_4_bytes()? != b"DIRC" {
            anyhow::bail!("Missing DIRC header");
        }

        let version = reader.read_4_bytes()?;
        if version != [0, 0, 0, 2] {
            anyhow::bail!("Invalid index: Unsupported index version: {version:?}");
        }

        let num_entries = reader.read_u32()?;

        let mut entries = Vec::<IndexEntry>::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let entry_start_pos = reader.reader.stream_position()?;

            let ctime = reader.read_timepair()?;
            let mtime = reader.read_timepair()?;
            let dev = reader.read_u32()?;
            let ino = reader.read_u32()?;
            let mode = reader.read_u32()?;
            let uid = reader.read_u32()?;
            let gid = reader.read_u32()?;
            let size = reader.read_u32()?;
            let sha = reader.read_allocate_n::<20>()?;

            // A 16-bit 'flags' field split into (high to low bits)
            // 1-bit assume-valid flag
            // 1-bit extended flag (must be zero in version 2)
            // 2-bit stage (during merge)
            // 12-bit name length if the length is less than 0xFFF; otherwise 0xFFF is stored in this field.
            let flags = reader.read_u16()?;
            let name_length = flags & 0xFFF;

            // version 3 has some stuff here

            let name = reader.read_string(name_length as usize)?;

            // 1-8 nul bytes as necessary to pad the entry to a multiple of eight bytes
            // while keeping the name NUL-terminated.
            let entry_end_pos = reader.reader.stream_position()?;
            // extra +1 for nul byte
            let entry_len = (entry_end_pos - entry_start_pos) + 1;
            let padding = (8 - (entry_len % 8)) % 8;
            reader.reader.seek_relative(
                i64::try_from(padding + 1).expect("Unable to convert usize to i64"),
            )?;

            entries.push(IndexEntry {
                ctime,
                mtime,
                dev,
                ino,
                mode,
                uid,
                gid,
                size,
                sha,
                flags,
                name,
            });
        }

        Ok(Index { entries })
    }

    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        // 30 is a super rough estimate of a filename length
        // e.g. src/module/submodule/file.rs is 28 bytes
        let capacity = (mem::size_of::<IndexEntry>() + 30) * self.entries.len();
        let mut buf = Vec::<u8>::with_capacity(capacity);

        buf.extend_from_slice(b"DIRC");
        buf.extend_from_slice(&[0, 0, 0, 2]);
        buf.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());

        for entry in self.entries.iter() {
            let start_pos = buf.len();
            buf.extend_from_slice(&entry.ctime.s.to_be_bytes());
            buf.extend_from_slice(&entry.ctime.ns.to_be_bytes());
            buf.extend_from_slice(&entry.mtime.s.to_be_bytes());
            buf.extend_from_slice(&entry.mtime.ns.to_be_bytes());
            buf.extend_from_slice(&entry.dev.to_be_bytes());
            buf.extend_from_slice(&entry.ino.to_be_bytes());
            buf.extend_from_slice(&entry.mode.to_be_bytes());
            buf.extend_from_slice(&entry.uid.to_be_bytes());
            buf.extend_from_slice(&entry.gid.to_be_bytes());
            buf.extend_from_slice(&entry.size.to_be_bytes());
            buf.extend_from_slice(&entry.sha);
            buf.extend_from_slice(&entry.flags.to_be_bytes());
            buf.extend_from_slice(entry.name.as_bytes());
            buf.extend_from_slice(b"\0");

            let padding = (8 - ((buf.len() - start_pos) % 8)) % 8;

            buf.extend(std::iter::repeat(0).take(padding));
        }

        Ok(buf)
    }

    pub fn write_to_disk(&self) -> anyhow::Result<()> {
        fs::write(repo_root()?.join(".bit/index"), self.serialize()?)
            .context("Failed to write new index to disk")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// The last time a file's metadata changed
    pub ctime: TimePair,
    /// The last time a file's data changed
    pub mtime: TimePair,
    /// The ID of device containing this file
    pub dev: u32,
    /// The file's inode number
    pub ino: u32,
    /// Mode (32 bits, upper 16 unused)
    ///
    /// ┌──────────────────┬───────────┬──────────┐
    /// │      15-12       │   11-9    │   8-0    │
    /// ├──────────────────┼───────────┼──────────┤
    /// │   object type    │   owner   │  perms   │
    /// └──────────────────┴───────────┴──────────┘
    ///
    /// object type (15-12): 0b1000 = regular file, 0b1010 = symlink, 0b1110 = gitlink
    /// owner        (11-9): setuid/setgid/sticky (always 0 in practice)
    /// perms         (8-0): 0o755 or 0o644 for files, 0o000 for symlinks/gitlinks
    pub mode: u32,
    /// User ID of owner
    pub uid: u32,
    /// Group ID of ownner
    pub gid: u32,
    /// Size of this object, in bytes
    pub size: u32,
    /// The object's SHA
    pub sha: [u8; 20],
    /// ┌─────┬──────────┬───────┬─────────────────────────────┐
    /// │ 15  │    14    │ 13-12 │            11-0             │
    /// ├─────┼──────────┼───────┼─────────────────────────────┤
    /// │ A-V │ extended │ stage │          name len           │
    /// └─────┴──────────┴───────┴─────────────────────────────┘
    ///
    /// assume-valid (15): skip worktree change detection
    /// extended     (14): if set, a second flags field follows the entry
    /// stage     (13-12): merge conflict stage (0 = normal, 1-3 = conflict)
    /// name len   (11-0): filename byte length, capped at 0xFFF
    pub flags: u16,
    pub name: String,
}

#[derive(Debug)]
struct IndexReader {
    reader: BufReader<fs::File>,
    buf: [u8; 4],
}

impl IndexReader {
    fn new(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            reader: BufReader::new(fs::File::open(path).with_context(|| {
                format!("Unable to open index file '{}'", path.to_string_lossy())
            })?),
            buf: [0u8; 4],
        })
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        self.reader.read_exact(&mut self.buf)?;
        Ok(u32::from_be_bytes(self.buf))
    }

    fn read_u16(&mut self) -> io::Result<u16> {
        self.reader.read_exact(&mut self.buf[..2])?;
        Ok(u16::from_be_bytes(self.buf[..2].try_into().unwrap()))
    }

    fn read_4_bytes(&mut self) -> io::Result<&[u8]> {
        self.reader.read_exact(&mut self.buf)?;
        Ok(&self.buf)
    }

    fn read_allocate_n<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut buf = [0u8; N];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_string(&mut self, length: usize) -> io::Result<String> {
        let mut buf = vec![0u8; length];
        self.reader.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).into())
    }

    fn read_timepair(&mut self) -> anyhow::Result<TimePair> {
        let s = self.read_u32()?;
        let ns = self.read_u32()?;

        Ok(TimePair { s, ns })
    }
}

#[derive(Debug, Clone)]
pub struct TimePair {
    pub s: u32,
    pub ns: u32,
}
