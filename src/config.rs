use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub user: Option<User>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub email: Option<String>,
    pub name: Option<String>,
}
