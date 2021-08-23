use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs::{read_to_string, write};
use std::io::ErrorKind::NotFound;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub account: Account,
    pub config: Others,
}

#[derive(Deserialize)]
pub struct Account {
    pub email: String,
    pub password: String,
    pub sq_ans: [String; 3],
}

#[derive(Deserialize)]
pub struct Others {
    pub offset: i64,
    pub auto_offset: bool,
    pub spread: usize,
    pub microsoft_auth: bool,
    pub gc_snipe: bool,
    pub change_skin: bool,
    pub skin_model: String,
    pub skin_path: String,
    pub name_queue: Vec<String>,
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self> {
        match read_to_string(&config_path) {
            Ok(s) => {
                let config: Self = toml::from_str(&s)?;
                Ok(config)
            }
            Err(e) if e.kind() == NotFound => {
                write(&config_path, get_default_config().as_bytes())?;
                bail!(
                    "{} not found, creating a new config file",
                    config_path.display()
                );
            }
            Err(e) => bail!(e),
        }
    }
}

fn get_default_config() -> String {
    r#"[account]
email = "test@example.com"
password = "123456789"
# Leave the strings in this array empty if your Minecraft account does not have security questions
sq_ans = ["Foo", "Bar", "Baz"]

[config]
offset = 0
auto_offset = false
# Spread (delay in milliseconds between each snipe request, not to be confused with offset which is the number of millseconds in which the sniper sends its first request before the name drops)
spread = 0
microsoft_auth = false
gc_snipe = false
change_skin = false
skin_model = "slim"
skin_path = "example.png"
# Name queueing (allows you to queue up multiple names for sniping)
# Note: This is an optional feature, leave this array empty if you prefer to enter your name manually via an input prompt)
name_queue = []

"#
    .to_string()
}
