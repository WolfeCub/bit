use std::{env, path::PathBuf};

pub fn repo_root() -> Option<PathBuf> {
    let mut cwd = env::current_dir().expect("Failed to get current directory");
    loop {
        if cwd.join(".bit").exists() {
            return Some(cwd);
        }

        if let Some(next) = cwd.parent() {
            cwd = next.to_path_buf();
        } else {
            return None;
        }
    }
}
