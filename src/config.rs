use crate::cli::pretty_panic;
use crate::constants::CONFIG_PATH;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct Config {
    pub account: Account,
    pub config: SubConfig,
}

#[derive(Deserialize)]
pub struct Account {
    pub username: String,
    pub password: String,
    pub sq1: String,
    pub sq2: String,
    pub sq3: String,
}

#[derive(Deserialize)]
pub struct SubConfig {
    pub auto_offset: bool,
    pub spread: u32,
    pub microsoft_auth: bool,
    pub gc_snipe: bool,
    pub change_skin: bool,
    pub skin_model: String,
    pub skin_filename: String,
}

impl Config {
    pub fn new() -> Self {
        match File::open(CONFIG_PATH) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s).unwrap();
                let config: Result<Self, _> = toml::from_str(&s);
                let config = match config {
                    Ok(c) => c,
                    Err(_) => pretty_panic(&format!(
                        "Error parsing {}, please check the formatting of the file.",
                        CONFIG_PATH
                    )),
                };
                if !(config.config.skin_model.to_lowercase() == "slim"
                    || config.config.skin_model.to_lowercase() == "classic")
                {
                    pretty_panic("Invalid skin type.");
                }
                config
            }
            Err(_) => pretty_panic(&format!("File {} not found.", CONFIG_PATH)),
        }
    }
}
