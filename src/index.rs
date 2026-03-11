use std::{
    fs,
    io::{self, BufReader, Read, Seek},
    path::Path,
};
use anyhow::anyhow;

use crate::util::repo_root;

#[derive(Debug)]
pub struct Index {
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn parse_from_disk() -> anyhow::Result<Self> {
        let path = repo_root()?.join(".bit/index");

        let mut reader = IndexReader::new(&path)?;

        if reader.read_4_bytes()? != b"DIRC" {
            return Err(anyhow!("Missing DIRC header"));
        }

        let version = reader.read_4_bytes()?;
        if version != [0, 0, 0, 2] {
            return Err(anyhow!("Invalid index: Unsupported index version: {version:?}"));
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
}

#[derive(Debug)]
pub struct IndexEntry {
    /// The last time a file's metadata changed
    pub ctime: TimePair,
    /// The last time a file's data changed
    pub mtime: TimePair,
    /// The ID of device containing this file
    pub dev: u32,
    /// The file's inode number
    pub ino: u32,
    // TODO:
    pub mode: u32,
    /// User ID of owner
    pub uid: u32,
    /// Group ID of ownner
    pub gid: u32,
    /// Size of this object, in bytes
    pub size: u32,
    /// The object's SHA
    pub sha: [u8; 20],
    // TODO:
    pub flags: u16,
    pub name: String,
}

#[derive(Debug)]
struct IndexReader {
    reader: BufReader<fs::File>,
    buf: [u8; 4],
}

impl IndexReader {
    fn new(path: &Path) -> io::Result<Self> {
        Ok(Self {
            reader: BufReader::new(fs::File::open(path)?),
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

#[derive(Debug)]
pub struct TimePair {
    pub s: u32,
    pub ns: u32,
}
