use std::{env, ffi::OsString, fs, path::{Path, PathBuf}, process::Command};

use chrono::{DateTime, Local};

use crate::errors::BitError;

pub fn repo_root() -> Result<PathBuf, BitError> {
    let mut cwd = env::current_dir()?;
    loop {
        if cwd.join(".bit").exists() {
            return Ok(cwd);
        }

        if let Some(next) = cwd.parent() {
            cwd = next.to_path_buf();
        } else {
            return Err(BitError::NotInRepo);
        }
    }
}

pub fn object_path(root: PathBuf, hash: &str) -> PathBuf {
    root.join(".bit/objects").join(&hash[..2]).join(&hash[2..])
}

pub fn parse_line<'a>(prefix: &[u8], body: &'a [u8]) -> (Option<String>, &'a [u8]) {
    let Some(rest) = body.strip_prefix(prefix) else {
        return (None, body);
    };

    let Some(eol) = rest.iter().position(|c| *c == b'\n') else {
        return (None, body);
    };

    let p = String::from_utf8(rest[..eol].to_vec()).ok();

    (p, &rest[eol + 1..])
}

pub fn git_time() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%s %z").to_string()
}

pub fn editor<P>(path: P, initial_content: &str) -> Result<String, BitError>
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
        Err(BitError::EmptyMessage("tag".into()))
    } else {
        Ok(filtered.join("\n"))
    }
}
