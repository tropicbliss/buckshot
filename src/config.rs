use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs::{read_to_string, write};
use std::io::ErrorKind;
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
    pub skin_path: PathBuf,
    pub slim: bool,
}

#[derive(Clone, Deserialize)]
pub struct Account {
    pub email: String,
    pub password: String,
    pub sq_ans: Option<[String; 3]>,
    pub giftcode: Option<String>,
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self> {
        let cfg_str = match read_to_string(&config_path) {
            Ok(x) => x,
            Err(y) if y.kind() == ErrorKind::NotFound => {
                let sample_cfg_u8 = include_bytes!("../config.toml");
                write(config_path, sample_cfg_u8)?;
                bail!(
                    "{} not found, creating a sample config file",
                    config_path.display()
                );
            }
            Err(z) => bail!(z),
        };
        let cfg: Self = toml::from_str(&cfg_str)?;
        Ok(cfg)
    }
}
