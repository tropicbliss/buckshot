use anyhow::Result;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct Config {
    pub account_entry: Vec<Account>,
    pub offset: Option<i64>,
    pub spread: usize,
    pub microsoft_auth: bool,
    pub gc_snipe: bool,
    pub skin: Option<Skin>,
    pub name_queue: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct Skin {
    pub skin_model: String,
    pub skin_path: PathBuf,
}

#[derive(Deserialize)]
pub struct Account {
    pub email: String,
    pub password: String,
    pub sq_ans: Option<[String; 3]>,
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self> {
        let s = read_to_string(&config_path)?;
        let config: Self = toml::from_str(&s)?;
        Ok(config)
    }
}
