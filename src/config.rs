use crate::cli::pretty_panic;
use crate::constants::CONFIG_PATH;
use serde::Deserialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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
    pub offset: i32,
    pub auto_offset: bool,
    pub spread: u32,
    pub microsoft_auth: bool,
    pub gc_snipe: bool,
    pub change_skin: bool,
    pub skin_model: String,
    pub skin_filename: String,
}

impl Config {
    pub async fn new(config_name: &Option<String>) -> Self {
        let config_path = if let Some(x) = config_name {
            x
        } else {
            CONFIG_PATH
        };
        match File::open(&config_path).await {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s).await.unwrap();
                let config: Result<Self, _> = toml::from_str(&s);
                let config = match config {
                    Ok(c) => c,
                    Err(_) => pretty_panic(&format!(
                        "Error parsing {}, please check the formatting of the file.",
                        config_path
                    )),
                };
                if !(config.config.skin_model.to_lowercase() == "slim"
                    || config.config.skin_model.to_lowercase() == "classic")
                {
                    pretty_panic("Invalid skin type.");
                }
                config
            }
            Err(_) => pretty_panic(&format!("File {} not found.", config_path)),
        }
    }
}
