use crate::config::Config;
use crate::constants;
use reqwest::blocking::Client;
use serde_json::{Result, Value};

pub struct Setup {
    config: Config,
    client: Client,
}

impl Setup {
    // Making a new Setup instance
    pub fn new(config: Config) -> Self {
        Setup {
            config: config,
            client: Client::new(),
        }
    }

    // Public facing function which doubles as a sniping implementation chooser for the setup process. Requests are synchronous atm for easy maintenance.
    pub fn setup(&self) {
        if !self.config.config.microsoft_auth {
            if self.config.config.gc_snipe {
                println!(
                    r#""microsoft_auth" is set to false yet "gc_snipe" is set to true. Defaulting to gift code sniping instead."#
                );
                self.gc();
            } else {
                self.mojang();
            }
        } else {
            if self.config.config.gc_snipe {
                self.gc();
            } else {
                self.msa();
            }
        }
    }

    // Code runner for setup of Mojang Sniper
    fn mojang(&self) {
        // code
    }

    // Code runner for setup of Microsoft Non-GC Sniper
    fn msa(&self) {
        // code
    }

    // Code runner for setup of Microsoft GC Sniper
    fn gc(&self) {
        // code
    }

    // The functions below are functions for handling reqwest requests and other miscellaneous tasks. Requests are blocking atm for easy maintenance.
    // Authenticator for Yggdrasil (Mojang)
    fn authenticate_mojang(&self) -> String {
        let post_body = format!(
            r#"{{"agent":{{"name":"Minecraft","version":1}},"username":"{}","password":"{}","clientToken":"Mojang-API-Client","requestUser":"true"}}"#,
            self.config.account.username, self.config.account.password
        );
        let url = format!("{}/authenticate", constants::YGGDRASIL_ORIGIN_SERVER);
        let res = self.client.post(url).body(post_body).send().unwrap();
        let status_code = res.status().as_u16();
        if status_code == 403 {
            panic!("[Authentication] Authentication error. Check if you have entered your username and password correctly.");
        }
        if status_code != 200 {
            panic!("[Authentication] HTTP status code: {}", status_code);
        }
        let body = res.text().unwrap();
        let json: Value = serde_json::from_str(&body).unwrap();
        String::from(json["accessToken"].as_str().unwrap())
    }
}

pub struct Sniper {
    setup: Setup,
    username_to_snipe: String,
    offset: i32,
}

impl Sniper {
    pub fn new(setup: Setup, username_to_snipe: String, offset: i32) -> Self {
        Sniper {
            setup: setup,
            username_to_snipe: username_to_snipe,
            offset: offset,
        }
    }

    // Public facing function which doubles as a sniping implementation chooser for the sniping process
    pub fn snipe(&self) {
        if !self.setup.config.config.microsoft_auth {
            if self.setup.config.config.gc_snipe {
                println!(
                    r#""microsoft_auth" is set to false yet "gc_snipe" is set to true. Defaulting to gift code sniping instead."#
                );
                self.gc();
            } else {
                self.mojang();
            }
        } else {
            if self.setup.config.config.gc_snipe {
                self.gc();
            } else {
                self.msa();
            }
        }
    }

    // Code runner for sniping routine of Mojang Sniper
    fn mojang(&self) {
        // code
    }
    // Code runner for sniping routine of Microsoft Non-GC Sniper
    fn msa(&self) {
        // code
    }
    // Code runner for sniping routine of Microsoft GC Sniper
    fn gc(&self) {
        // code
    }
}
