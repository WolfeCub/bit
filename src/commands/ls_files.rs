use crate::{
    objects::{Index, IndexEntry}, utils::{cwd, relative_path_string, repo_root},
};
use chrono::{DateTime, Local};
use clap::Args;
use std::time::{Duration, UNIX_EPOCH};

use colored::Colorize;

#[derive(Args, Debug)]
pub struct LsFilesArg {
    /// This doesn't exist in actual git but it's useful for inspecting our index
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

impl LsFilesArg {
    pub fn run(self) -> anyhow::Result<()> {
        let root = repo_root()?;
        let index = Index::parse_from_disk()?;

        let cwd = cwd()?;
        for entry in &index.entries {
            let relative = relative_path_string(root.join(&entry.name), &cwd);
            println!("{}", relative);
            if self.verbose {
                print_verbose(entry);
            }
        }

        Ok(())
    }
}

fn print_verbose(entry: &IndexEntry) {
    let mode_type = (entry.mode >> 12) as u8;
    let mode_perms = entry.mode & 0o777;

    let entry_type = match mode_type {
        0b1000 => "regular file",
        0b1010 => "symlink",
        0b1110 => "git link",
        _ => "unknown",
    };

    let sha_hex = entry
        .sha
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let perms = format!("{:o}", mode_perms);
    println!("  {} with perms: {}", entry_type.yellow(), perms.yellow());
    println!("  on blob: {}", sha_hex.cyan());
    println!("  size: {}", entry.size.to_string().green());
    println!(
        "  created: {}, modified: {}",
        format_timepair(entry.ctime.s, entry.ctime.ns).blue(),
        format_timepair(entry.mtime.s, entry.mtime.ns).blue(),
    );
    println!(
        "  device: {}, inode: {}",
        entry.dev.to_string().magenta(),
        entry.ino.to_string().magenta()
    );
    println!(
        "  user: {}  group: {}",
        entry.uid.to_string().bold(),
        entry.gid.to_string().bold()
    );

    let flag_stage = (entry.flags >> 12) & 0x3;
    let flag_assume_valid = (entry.flags >> 15) & 0x1;
    println!(
        "  flags: stage={} assume_valid={}",
        flag_stage.to_string().bold(), flag_assume_valid.to_string().bold()
    );

    println!();
}

fn format_timepair(s: u32, ns: u32) -> String {
    let system_time = UNIX_EPOCH + Duration::new(s as u64, ns);
    let datetime: DateTime<Local> = system_time.into();
    format!("{}.{}", datetime.format("%Y-%m-%d %H:%M:%S"), ns)
}

