#![allow(clippy::struct_excessive_bools)]

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
    pub sq1: String,
    pub sq2: String,
    pub sq3: String,
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
    pub skin_filename: String,
    pub name_queue: Vec<String>,
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self> {
        match read_to_string(&config_path) {
            Ok(s) => {
                let config: Self = toml::from_str(&s)?;
                if !(config.config.skin_model.to_lowercase() == "slim"
                    || config.config.skin_model.to_lowercase() == "classic")
                {
                    bail!("Invalid skin variant");
                }
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
password = "test"
# Leave the rest empty if you do not have security questions
sq1 = "Foo"
sq2 = "Bar"
sq3 = "Baz"

[config]
offset = 0
auto_offset = false
# Spread (if you are unsure leave it as it is)
spread = 0
microsoft_auth = false
gc_snipe = false
change_skin = false
skin_model = "slim"
skin_filename = "example.png"
# Name queueing (allows you to snipe multiple names sequentially)
# Note: This is an optional feature, leave it as it is if you only want to snipe one name
# Example:
# name_queue = ["Marc", "Dream"]
name_queue = []
"#
    .to_string()
}
