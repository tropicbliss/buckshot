use crate::constants;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Deserialize)]
pub struct Config {
    pub account_entry: Vec<Account>,
    pub offset: u32,
    pub spread: u32,
    pub microsoft_auth: bool,
    pub gc_snipe: bool,
    pub skin: Option<Skin>,
    pub name_queue: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Skin {
    pub file: bool,
    pub path: String,
    pub slim: bool,
}

#[derive(Clone, Deserialize)]
pub struct Account {
    pub email: String,
    pub password: String,
    pub sq_ans: Option<[String; 3]>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let cfg = read_to_string(constants::CONFIG_PATH)?;
        let cfg: Self = toml::from_str(&cfg)?;
        if cfg.account_entry.is_empty() {
            bail!("No accounts provided in config file");
        }
        if let Some(count) = &cfg.name_queue {
            if count.is_empty() {
                bail!("No name provided in name queue");
            }
        }
        Ok(cfg)
    }
}
