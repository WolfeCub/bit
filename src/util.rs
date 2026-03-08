use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use cached::proc_macro::cached;
use chrono::{DateTime, Local};

use crate::{config::Config, errors::BitError};

#[cached(result = true)]
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

pub fn object_path(hash: &str) -> Result<PathBuf, BitError> {
    Ok(repo_root()?
        .join(".bit/objects")
        .join(&hash[..2])
        .join(&hash[2..]))
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

// TODO: Local repo config
pub fn get_config() -> Option<Config> {
    let home = env::home_dir()?;
    let config_path = home.join(".gitconfig");
    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        serini::from_str(&content).ok()
    } else {
        None
    }
}

pub fn get_user_info() -> (String, String) {
    let (name, email) = get_config()
        .and_then(|c| c.user)
        .map(|u| (u.name, u.email))
        .unwrap_or((None, None));

    let resolved_name = name.unwrap_or_else(|| {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    });

    let resolved_email = email.unwrap_or_else(|| {
        let e = fs::read_to_string("/etc/hostname")
            .or_else(|_| env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        format!("{}@{}", resolved_name, e.trim())
    });

    (resolved_name, resolved_email)
}

// TODO: Support globs and directories
pub fn is_file_ignored(file: &str) -> bool {
    ignore_patterns().iter().any(|p| p == file)
}

#[cached]
pub fn ignore_patterns() -> Vec<String> {
    let Ok(root) = repo_root() else {
        return vec![];
    };
    let Ok(contents) = fs::read_to_string(root.join(".bitignore")) else {
        return vec![];
    };

    contents
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}
