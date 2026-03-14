use std::{env, fs};

use crate::objects::Config;

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


