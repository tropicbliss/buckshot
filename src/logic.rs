use crate::constants;
use reqwest;
use reqwest::header;
use reqwest::Error;
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
    // Opens and deserialises config.toml and maps the options to Sniper struct
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
    // Public facing function which doubles as a sniping implementation chooser for the setup process
    pub fn setup(&self, client: reqwest::Client) {
        if !self.config.microsoft_auth {
            if self.config.gc_snipe {
                println!(
                    r#""microsoft_auth" is set to false yet "gc_snipe" is set to true. Defaulting to gift code sniping instead."#
                );
                self.setup_gc(client);
            } else {
                self.setup_mojang(client);
            }
        } else {
            if self.config.gc_snipe {
                self.setup_gc(client);
            } else {
                self.setup_msa(client);
            }
        }
    }
    // Public facing function which doubles as a sniping implementation chooser for the sniping process
    pub fn snipe(&self, username_to_snipe: String, offset: i32, client: reqwest::Client) {
        if !self.config.microsoft_auth {
            if self.config.gc_snipe {
                println!(
                    r#""microsoft_auth" is set to false yet "gc_snipe" is set to true. Defaulting to gift code sniping instead."#
                );
                self.snipe_gc(username_to_snipe, offset, client);
            } else {
                self.snipe_mojang(username_to_snipe, offset, client);
            }
        } else {
            if self.config.gc_snipe {
                self.snipe_gc(username_to_snipe, offset, client);
            } else {
                self.snipe_msa(username_to_snipe, offset, client);
            }
        }
    }
    // Code runner for setup of Mojang Sniper
    fn setup_mojang(&self, client: reqwest::Client) {
        // code
    }
    // Code runner for setup of Microsoft Non-GC Sniper
    fn setup_msa(&self, client: reqwest::Client) {
        // code
    }
    // Code runner for setup of Microsoft GC Sniper
    fn setup_gc(&self, client: reqwest::Client) {
        // code
    }
    // Code runner for sniping routine of Mojang Sniper
    fn snipe_mojang(&self, username_to_snipe: String, offset: i32, client: reqwest::Client) {
        // code
    }
    // Code runner for sniping routine of Microsoft Non-GC Sniper
    fn snipe_msa(&self, username_to_snipe: String, offset: i32, client: reqwest::Client) {
        // code
    }
    // Code runner for sniping routine of Microsoft GC Sniper
    fn snipe_gc(&self, username_to_snipe: String, offset: i32, client: reqwest::Client) {
        // code
    }
    // The functions below are functions for handling reqwest requests and other miscellaneous tasks. Requests are synchronous atm for easy maintenance.
    // Authenticator for Yggdrasil (Mojang)
    fn authenticate_mojang(&self, client: reqwest::Client) -> String {
        let body = format!("{\"agent\":{\"name\":\"Minecraft\",\"version\":1},\"username\":\"{}\",\"password\":\"{}\",\"clientToken\":\"Mojang-API-Client\",\"requestUser\":\"true\"}");
        let res = client
            .post(format!(
                "{}/authenticate",
                constants::YGGDRASIL_ORIGIN_SERVER
            ))
            .body(body)
            .send()
            .await?;
    }
}
