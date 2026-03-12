use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, anyhow};
use cached::proc_macro::cached;
use chrono::{DateTime, Local};

use crate::{commands::show_ref::resolve_ref, objects::Config};

#[cached(result = true)]
pub fn repo_root() -> anyhow::Result<PathBuf> {
    let mut cwd = env::current_dir()?;
    loop {
        if cwd.join(".bit").exists() {
            return Ok(cwd);
        }

        if let Some(next) = cwd.parent() {
            cwd = next.to_path_buf();
        } else {
            return Err(anyhow!(
                "Not a bit repository (or any of the parent directories)"
            ));
        }
    }
}

pub fn object_path(hash: &str) -> anyhow::Result<PathBuf> {
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
        Err(anyhow!("Empty tag message"))
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

pub fn find_hash(target: &str) -> anyhow::Result<String> {
    // Min length for a shortened hash is 4
    if target.len() >= 4 {
        let dir = repo_root()?.join(".bit/objects").join(&target[..2]);
        let entries = fs::read_dir(dir)
            .map(|rd| rd.collect::<Vec<_>>())
            .unwrap_or_default();

        // If there's exactly one hash and it's prefix matches target return it
        if let [Ok(e)] = entries.as_slice() {
            let file_name = e.file_name();
            let file_name = file_name.to_string_lossy();

            if file_name.starts_with(&target[2..]) {
                // Reconstruct the full hash
                return Ok(format!("{}{}", &target[..2], file_name));
            }
        }
    }

    // Priority list of search paths
    // i.e. refs/tags/name works but so does tags/name and just name
    // "" is repo root this covers things like: HEAD, ORIG_HEAD, MERGE_HEAD, etc
    // TODO: Currently the root is too loosey goosey `rev-parse COMMIT_EDITMSG` will just dump the file
    ["", "refs/", "refs/tags/", "refs/heads/", "refs/remotes/"]
        .iter()
        .find_map(|prefix| resolve_ref(&format!("{}{}", prefix, target)).ok())
        .context("Unable to resolve or ambiguous hash or ref")
}

pub fn normalize_paths(paths: &[String]) -> anyhow::Result<Vec<String>> {
    let root = repo_root()?;

    let cwd = env::current_dir()?;
    paths
        .iter()
        .map(|path| -> anyhow::Result<String> {
            let absolute_path = cwd.join(path).canonicalize()?;
            let repo_relative_path = absolute_path
                .strip_prefix(&root)
                .with_context(|| format!("Path {absolute_path:?} is not within the repository"))?;

            Ok(repo_relative_path.to_string_lossy().into())
        })
        .collect::<anyhow::Result<Vec<String>>>()
}
