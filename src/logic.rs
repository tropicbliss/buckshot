use crate::constants;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct Sniper {
    pub account: Account,
    pub config: Config,
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
pub struct Config {
    pub auto_offset: bool,
    pub spread: u32,
    pub microsoft_auth: bool,
    pub gc_snipe: bool,
    pub change_skin: bool,
    pub skin_model: String,
    pub skin_filename: String,
}

impl Sniper {
    pub fn new() -> Self {
        match File::open(constants::CONFIG_PATH) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s).unwrap();
                let config: Result<Sniper, toml::de::Error> = toml::from_str(&s);
                match config {
                    Ok(x) => x,
                    Err(_) => panic!("Cannot parse {}.", constants::CONFIG_PATH),
                }
            }
            Err(_) => panic!("File {} not found.", constants::CONFIG_PATH),
        }
    }
    pub fn setup(&self) {
        // code
    }
    pub fn snipe(&self, username_to_snipe: String, offset: i32) {
        // code
    }
}
