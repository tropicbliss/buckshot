use crate::cli::get_giftcode;
use crate::config::Config;
use crate::constants;
use chrono::{offset::Utc, Local, NaiveDateTime, TimeZone};
use reqwest::blocking::Client;
use serde_json::Value;
use spin_sleep;
use std::{thread, time};

pub struct Setup {
    config: Config,
    client: Client,
}

impl Setup {
    // Making a new Setup instance
    pub fn new(config: Config) -> Self {
        Self {
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
        if self.is_security_questions_needed(&access_token) {
            match self.get_security_questions_id(&access_token) {
                Some(x) => self.send_security_questions(x, &access_token),
                None => (),
            }
        }
        self.name_change_eligibility_checker(&access_token);
    }

    // Code runner for setup of Microsoft Non-GC Sniper
    fn msa(&self) {
        self.name_change_eligibility_checker(&self.authenticate_msa());
    }

    // Code runner for setup of Microsoft GC Sniper
    fn gc(&self) {
        match get_giftcode() {
            Some(x) => {
                self.redeem_giftcode(&self.authenticate_msa());
            }
            None => (),
        }
    }

    // The functions below are functions for handling reqwest requests and other miscellaneous tasks. Requests are blocking atm for easy maintenance.
    // Authenticator for Yggdrasil (Mojang)
    fn authenticate_mojang(&self) -> String {
        if self.config.account.username.is_empty() || self.config.account.password.is_empty() {
            panic!(
                "[ParseAccountFile] The username or password field in {} is empty.",
                constants::CONFIG_PATH
            );
        }
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
            er => panic!("[Authentication] HTTP status code: {}", er),
        }
    }

    fn is_security_questions_needed(&self, token: &str) -> bool {
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self.client.get(url).bearer_auth(token).send().unwrap();
        match res.status().as_u16() {
            204 => false,
            403 => true,
            er => panic!("[SecurityQuestionsCheck] HTTP status code: {}", er),
        }
    }

    fn get_security_questions_id(&self, token: &str) -> Option<[u64; 3]> {
        let url = format!("{}/user/security/challenges", constants::MOJANG_API_SERVER);
        let res = self.client.get(url).bearer_auth(token).send().unwrap();
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
            er => panic!("[GetSecurityQuestions] HTTP status code: {}", er),
        }
    }

    fn send_security_questions(&self, question_id_array: [u64; 3], token: &str) {
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
            .bearer_auth(token)
            .send()
            .unwrap();
        match res.status().as_u16() {
            403 => panic!("[SendSecurityQuestions] Authentication error. Check if you have entered your security questions correctly."),
            204 => (),
            er => panic!("[SendSecurityQuestions] HTTP status code: {}", er),
        }
    }

    fn name_change_eligibility_checker(&self, token: &str) {
        let url = format!(
            "{}/minecraft/profile/namechange",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self.client.get(url).bearer_auth(token).send().unwrap();
        match res.status().as_u16() {
            200 => {
                let body = res.text().unwrap();
                let json: Value = serde_json::from_str(&body).unwrap();
                if !json["nameChangeAllowed"].as_bool().unwrap() {
                    panic!("[NameChangeEligibilityChecker] You cannot name change within the cooldown period.");
                }
            }
            er => panic!("[NameChangeEligibilityChecker] HTTP status code: {}", er),
        }
    }

    fn authenticate_msa(&self) -> String {
        // TODO: MS auth flow.
        String::from("Bonjour")
    }

    fn redeem_giftcode(&self, token: &str) {
        let url = format!("{}/productvoucher", constants::MINECRAFTSERVICES_API_SERVER);
        let res = self.client.put(url).bearer_auth(token).send().unwrap();
        match res.status().as_u16() {
            200 => {
                println!("Gift code redeemed successfully.");
            }
            er => panic!("[GiftCodeRedemption] HTTP status code: {}", er),
        }
    }
}

pub struct Sniper {
    setup: Setup,
    username_to_snipe: String,
    offset: i32,
}

impl Sniper {
    pub fn new(setup: Setup, username_to_snipe: String, offset: i32) -> Self {
        Self {
            setup: setup,
            username_to_snipe: username_to_snipe,
            offset: offset,
        }
    }

    // Public facing function which doubles as a sniping implementation chooser for the sniping process
    pub fn snipe(&self) {
        self.is_name_available();
        self.execute(self.check_name_availability_time());
    }

    fn is_name_available(&self) {
        let url = format!(
            "{}/user/profile/agent/minecraft/name",
            constants::MOJANG_API_SERVER
        );
        let res = self.setup.client.get(url).send().unwrap();
        match res.status().as_u16() {
            204 => (),
            200 => panic!("[NameAvailabilityChecker] Name has been taken."),
            er => panic!("[NameAvailabilityChecker] HTTP status code: {}", er),
        }
    }

    fn check_name_availability_time(&self) -> i64 {
        let url = format!("{}/api/namemc/droptime", constants::KQZZ_NAMEMC_API);
        let res = self.setup.client.get(url).send().unwrap();
        match res.status().as_u16() {
            200 => {
                let body = res.text().unwrap();
                let json: Value = serde_json::from_str(&body).unwrap();
                json["droptime"].as_i64().unwrap()
            }
            er => panic!("[CheckNameAvailabilityTime] HTTP status code: {}", er),
        }
    }

    fn execute_mojang(&self, droptime_epoch: i64) {
        let droptime = NaiveDateTime::from_timestamp(droptime_epoch, 0);
        let local_droptime = Local.from_local_datetime(&droptime).unwrap();
        let epoch_now = Utc::now().naive_utc().timestamp();
        let duration_in_sec = droptime_epoch - epoch_now;
        if duration_in_sec < 60 {
            println!(
                "Sniping {} in ~{} seconds | sniping at {}",
                self.username_to_snipe,
                duration_in_sec,
                local_droptime.format("%F %T")
            );
        } else {
            println!(
                "Sniping {} in ~{} seconds | sniping at {}",
                self.username_to_snipe,
                duration_in_sec / 60,
                local_droptime.format("%F %T")
            );
        }
        // Setup up snipe request via socket api
        let setup_epoch = droptime_epoch - 20;
        if Utc::now().timestamp() < setup_epoch {
            thread::sleep(time::Duration::from_secs(
                (setup_epoch - Utc::now().timestamp()) as u64,
            ))
            // Do stuff
        }
        println!("Signed in to {}.", self.setup.config.account.username);
        println!("Setup complete!");
        spin_sleep::sleep(time::Duration::from_secs(
            (droptime_epoch - Utc::now().timestamp()) as u64,
        ));
    }
    // Snipe
    // Change skin if successful
}

pub fn auto_offset_calculation() -> i32 {
    // Use sockets
    23
}
