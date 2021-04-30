use crate::config::Config;
use crate::constants;
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::io;
use std::{thread, time};
use webbrowser;

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
        let access_token = self.authenticate_mojang();
        let auth_header = self.authheader_builder(access_token);
        if self.is_security_questions_needed(auth_header.clone()) {
            match self.get_security_questions_id(auth_header.clone()) {
                Some(x) => self.send_security_questions(x, auth_header.clone()),
                None => (),
            }
        }
        self.name_change_eligibility_checker(auth_header);
    }

    // Code runner for setup of Microsoft Non-GC Sniper
    fn msa(&self) {
        let access_token = self.authenticate_msa();
        let auth_header = self.authheader_builder(access_token);
        self.name_change_eligibility_checker(auth_header);
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
        match res.status().as_u16() {
            403 => panic!("[Authentication] Authentication error. Check if you have entered your username and password correctly."),
            200 => {
                let body = res.text().unwrap();
                let json: Value = serde_json::from_str(&body).unwrap();
                String::from(json["accessToken"].as_str().unwrap())
            },
            er => panic!(format!("[Authentication] HTTP status code: {}", er)),
        }
    }

    fn is_security_questions_needed(&self, header: HeaderMap) -> bool {
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self.client.get(url).headers(header).send().unwrap();
        match res.status().as_u16() {
            204 => false,
            403 => true,
            er => panic!(format!("[SecurityQuestionsCheck] HTTP status code: {}", er)),
        }
    }

    fn get_security_questions_id(&self, header: HeaderMap) -> Option<[u64; 3]> {
        let url = format!("{}/user/security/challenges", constants::MOJANG_API_SERVER);
        let res = self.client.get(url).headers(header).send().unwrap();
        match res.status().as_u16() {
            200 => {
                let body = res.text().unwrap();
                if body.eq("[]") {
                    None
                } else {
                    let json_array: Value = serde_json::from_str(&body).unwrap();
                    let first = json_array[0]["answer"]["id"].as_u64().unwrap();
                    let second = json_array[1]["answer"]["id"].as_u64().unwrap();
                    let third = json_array[2]["answer"]["id"].as_u64().unwrap();
                    Some([first, second, third])
                }
            }
            er => panic!(format!("[GetSecurityQuestions] HTTP status code: {}", er)),
        }
    }

    fn send_security_questions(&self, question_id_array: [u64; 3], header: HeaderMap) {
        let post_body = format!(
            r#"[{{"id":{},"answer":"{}"}},{{"id":{},"answer":"{}"}},{{"id":{},"answer":"{}"}}]"#,
            question_id_array[0],
            self.config.account.sq1,
            question_id_array[1],
            self.config.account.sq2,
            question_id_array[2],
            self.config.account.sq3
        );
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .post(url)
            .body(post_body)
            .headers(header)
            .send()
            .unwrap();
        match res.status().as_u16() {
            403 => panic!("[SendSecurityQuestions] Authentication error. Check if you have entered your security questions correctly."),
            204 => (),
            er => panic!(format!("[SendSecurityQuestions] HTTP status code: {}", er)),
        }
    }

    fn name_change_eligibility_checker(&self, header: HeaderMap) {
        let url = format!(
            "{}/minecraft/profile/namechange",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self.client.get(url).headers(header).send().unwrap();
        match res.status().as_u16() {
            200 => {
                let body = res.text().unwrap();
                let json: Value = serde_json::from_str(&body).unwrap();
                if !json["nameChangeAllowed"].as_bool().unwrap() {
                    panic!("[NameChangeEligibilityChecker] You cannot name change within the cooldown period.");
                }
            }
            er => panic!(format!(
                "[NameChangeEligibilityChecker] HTTP status code: {}",
                er
            )),
        }
    }

    fn authenticate_msa(&self) -> String {
        println!("Opening browser...");
        // Gives the user an illusion that something is happening.
        thread::sleep(time::Duration::from_secs(3));
        match webbrowser::open("https://login.live.com/oauth20_authorize.srf?client_id=9abe16f4-930f-4033-b593-6e934115122f&response_type=code&redirect_uri=https%3A%2F%2Fapi.gosnipe.tech%2Fapi%2Fauthenticate&scope=XboxLive.signin%20XboxLive.offline_access") {
            Ok(_) => (),
            Err(_) => {
                println!("Looks like you are running this program in a headless environment. Copy the following URL into your browser:");
                println!("https://login.live.com/oauth20_authorize.srf?client_id=9abe16f4-930f-4033-b593-6e934115122f&response_type=code&redirect_uri=https%3A%2F%2Fapi.gosnipe.tech%2Fapi%2Fauthenticate&scope=XboxLive.signin%20XboxLive.offline_access");
            },
        }
        println!("Please make sure that your snipe will not last more than a day or the snipe will fail.");
        let mut input = String::new();
        print!(
            r#"Sign in with your Microsoft account and copy the ID from the "access_token" field right here: "#
        );
        io::Write::flush(&mut io::stdout()).unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        input.to_string()
    }

    fn authheader_builder(&self, token: String) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let token = format!("Bearer {}", token);
        let mut auth_value = HeaderValue::from_str(&token).unwrap();
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        headers
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
