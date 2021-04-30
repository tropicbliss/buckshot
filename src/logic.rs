use crate::config::Config;
use crate::constants;
use reqwest;

pub struct Setup {
    config: Config,
    client: reqwest::Client,
}

impl Setup {
    // Making a new Setup instance
    pub fn new(config: Config) -> Self {
        Setup {
            config: config,
            client: reqwest::Client::new(),
        }
    }

    // Public facing function which doubles as a sniping implementation chooser for the setup process
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

    // The functions below are functions for handling reqwest requests and other miscellaneous tasks.
    // Authenticator for Yggdrasil (Mojang)
    fn authenticate_mojang(&self) -> String {
        let post_body = format!(
            r#"{{"agent":{{"name":"Minecraft","version":1}},"username":"{}","password":"{}","clientToken":"Mojang-API-Client","requestUser":"true"}}"#,
            self.config.account.username, self.config.account.password
        );
        let url = format!("{}/authenticate", constants::YGGDRASIL_ORIGIN_SERVER);
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
