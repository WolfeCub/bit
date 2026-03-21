pub mod bit_dir_walker;
pub mod changes;
pub mod config;
pub mod diff;
pub mod head;
pub mod path;
pub mod repo;

use std::{fs, io::Write, path::Path, process::{Command, Stdio}};

use anyhow::anyhow;
use chrono::{DateTime, Local};

pub fn parse_field<'a>(prefix: &[u8], body: &'a [u8]) -> (Option<String>, &'a [u8]) {
    let Some(rest) = body.strip_prefix(prefix) else {
        return (None, body);
    };

    let n: usize = rest
        .split(|c| *c == b'\n')
        .enumerate()
        .take_while(|(i, line)| *i == 0 || matches!(line.first(), Some(b' ' | b'\t')))
        .map(|(_, l)| l.len() + 1)
        .sum();

    let field = String::from_utf8(rest[..n].trim_ascii_end().to_vec()).ok();
    (field, &rest[n..])
}

pub fn git_time() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%s %z").to_string()
}

pub fn editor<P>(path: P, initial_content: &str) -> anyhow::Result<String>
where
    P: AsRef<Path>,
    P: AsRef<std::ffi::OsStr>,
{
    fs::write(&path, initial_content)?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    Command::new(editor).arg(&path).status()?;

    let content = fs::read_to_string(&path)?;

    let filtered = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        Err(anyhow!("Empty message"))
    } else {
        Ok(filtered.join("\n"))
    }
}

// TODO: Support custom pagers
pub fn pager(content: &str) -> anyhow::Result<()> {
    let mut child = Command::new("less")
        .arg("-FRX")
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    child.wait()?;

    Ok(())
}
