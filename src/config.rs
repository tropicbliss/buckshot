use crate::cli::pretty_panik;
use crate::constants::DEFAULT_CONFIG_PATH;
use serde::Deserialize;
use std::io::ErrorKind::NotFound;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    pub name_queue: Vec<String>,
}

impl Config {
    pub async fn new(config_name: &Option<String>) -> Self {
        let function_id = "ConfigNew";
        let config_path = match config_name {
            Some(x) => x,
            None => DEFAULT_CONFIG_PATH,
        };
        match File::open(&config_path).await {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s).await.unwrap();
                let config: Result<Self, _> = toml::from_str(&s);
                let config = match config {
                    Ok(c) => c,
                    Err(e) => pretty_panik(
                        function_id,
                        &format!("Error parsing {}. Reason: {}.", config_path, e),
                    ),
                };
                if !(config.config.skin_model.to_lowercase() == "slim"
                    || config.config.skin_model.to_lowercase() == "classic")
                {
                    pretty_panik(function_id, "Invalid skin type.");
                }
                config
            }
            Err(e) if e.kind() == NotFound => {
                let path = Path::new(config_path);
                let mut file = match File::create(path).await {
                    Ok(x) => x,
                    Err(e) => pretty_panik(
                        function_id,
                        &format!(
                            "File {} not found, a new config file cannot be created. Reason: {}.",
                            config_path, e
                        ),
                    ),
                };
                file.write_all(&get_default_config().into_bytes())
                    .await
                    .unwrap();
                pretty_panik(function_id, &format!(
                    "File {} not found, creating a new config file. Please enter any relevant information to the file.",
                    config_path
                ));
            }
            Err(e) => panic!("{}", e),
        }
    }
}

fn get_default_config() -> String {
    r#"[account]
username = "test@example.com"
password = "test"
# Leave the rest empty if you do not have security questions
sq1 = "Foo"
sq2 = "Bar"
sq3 = "Baz"

[config]
offset = 0
auto_offset = false
spread = 0
microsoft_auth = false
gc_snipe = false
change_skin = false
skin_model = "slim"
skin_filename = "example.png"
# Name queueing example:
# name_queue = ["Marc", "Dream"]
name_queue = []
"#
    .to_string()
}
