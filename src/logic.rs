use crate::cli;
use crate::config::Config;
use crate::constants;
use crate::socket;
use chrono::{offset::Utc, Local, NaiveDateTime, TimeZone};
use reqwest::blocking::Client;
use serde_json::Value;
use spin_sleep;
use std::{thread, time};

pub struct Sniper {
    config: Config,
    client: Client,
}

impl Sniper {
    // Making a new Setup instance
    pub fn new(config: Config) -> Self {
        Self {
            config: config,
            client: Client::new(),
        }
    }

    // Public facing function which doubles as a sniping implementation chooser for the setup process. Requests are synchronous atm for easy maintenance.
    pub fn snipe(&self) {
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
        let username = cli::get_username_choice();
        let offset;
        if self.config.config.auto_offset {
            offset = self.auto_offset_calculation(&username);
        } else {
            offset = cli::get_offset();
        }
        self.is_name_available();
        self.execute_mojang(
            self.check_name_availability_time(),
            &username,
            offset,
            access_token,
        );
    }

    // Code runner for setup of Microsoft Non-GC Sniper
    fn msa(&self) {
        let token = self.authenticate_msa();
        self.name_change_eligibility_checker(&token[0]);
        let username = cli::get_username_choice();
        let offset;
        if self.config.config.auto_offset {
            offset = self.auto_offset_calculation(&username);
        } else {
            offset = cli::get_offset();
        }
        self.is_name_available();
    }

    // Code runner for setup of Microsoft GC Sniper
    fn gc(&self) {
        let token = self.authenticate_msa();
        match cli::get_giftcode() {
            Some(x) => {
                self.redeem_giftcode(&token[0], x);
            }
            None => (),
        }
        let username = cli::get_username_choice();
        let offset;
        if self.config.config.auto_offset {
            offset = self.auto_offset_calculation(&username);
        } else {
            offset = cli::get_offset();
        }
        self.is_name_available();
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

    fn authenticate_msa(&self) -> [String; 2] {
        println!("Opening browser...");
        // Gives the user an illusion that something is happening.
        thread::sleep(time::Duration::from_secs(3));
        match webbrowser::open(constants::MS_AUTH_SERVER) {
            Ok(_) => (),
            Err(_) => {
                println!("Looks like you are running this program in a headless environment. Copy the following URL into your browser:");
                println!("{}", constants::MS_AUTH_SERVER);
            }
        }
        let access_token = cli::get_access_token();
        let refresh_token = cli::get_refresh_token();
        [access_token, refresh_token]
    }

    fn redeem_giftcode(&self, token: &str, giftcode: String) {
        let url = format!(
            "{}/productvoucher/{}",
            constants::MINECRAFTSERVICES_API_SERVER,
            giftcode
        );
        let res = self.client.put(url).bearer_auth(token).send().unwrap();
        match res.status().as_u16() {
            200 => {
                println!("Gift code redeemed successfully.");
            }
            er => panic!("[GiftCodeRedemption] HTTP status code: {}", er),
        }
    }

    fn auto_offset_calculation(&self, username: &str) -> i32 {
        let payload = vec![format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n", username).as_bytes()];
        let mut conn = socket::TLSConnectionManager::new(
            "api.minecraftservices.com".to_string(),
            443,
            "api.minecraftservices.com".to_string(),
        );
        conn.connect(1);
        let before = Utc::now();
        conn.send(payload);
        Utc::now().signed_duration_since(before).num_milliseconds() as i32
            - constants::SERVER_RESPONSE_DURATION
    }

    fn is_name_available(&self) {
        let url = format!(
            "{}/user/profile/agent/minecraft/name",
            constants::MOJANG_API_SERVER
        );
        let res = self.client.get(url).send().unwrap();
        match res.status().as_u16() {
            204 => (),
            200 => panic!("[NameAvailabilityChecker] Name has been taken."),
            er => panic!("[NameAvailabilityChecker] HTTP status code: {}", er),
        }
    }

    fn check_name_availability_time(&self) -> i64 {
        let url = format!("{}/api/namemc/droptime", constants::KQZZ_NAMEMC_API);
        let res = self.client.get(url).send().unwrap();
        match res.status().as_u16() {
            200 => {
                let body = res.text().unwrap();
                let json: Value = serde_json::from_str(&body).unwrap();
                json["droptime"].as_i64().unwrap()
            }
            er => panic!("[CheckNameAvailabilityTime] HTTP status code: {}", er),
        }
    }

    fn execute_mojang(
        &self,
        droptime_epoch: i64,
        username_to_snipe: &str,
        offset: i32,
        mut access_token: String,
    ) {
        let droptime = NaiveDateTime::from_timestamp(droptime_epoch, 0);
        let local_droptime = Local.from_local_datetime(&droptime).unwrap();
        let epoch_now = Utc::now().naive_utc().timestamp();
        let duration_in_sec = droptime_epoch - epoch_now;
        if duration_in_sec < 60 {
            println!(
                "Sniping {} in ~{} seconds | sniping at {}",
                username_to_snipe,
                duration_in_sec,
                local_droptime.format("%F %T")
            );
        } else {
            println!(
                "Sniping {} in ~{} seconds | sniping at {}",
                username_to_snipe,
                duration_in_sec / 60,
                local_droptime.format("%F %T")
            );
        }
        let setup_epoch = droptime_epoch - 20;
        if Utc::now().timestamp() < setup_epoch {
            thread::sleep(time::Duration::from_secs(
                (setup_epoch - Utc::now().timestamp() - offset as i64) as u64,
            ));
            access_token = self.authenticate_mojang();
            if self.is_security_questions_needed(&access_token) {
                match self.get_security_questions_id(&access_token) {
                    Some(x) => self.send_security_questions(x, &access_token),
                    None => (),
                }
            }
            self.name_change_eligibility_checker(&access_token);
            self.is_name_available();
        }
        let payload = vec![format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n", username_to_snipe, access_token).as_bytes()];
        let mut conn_vec: Vec<socket::TLSConnectionManager> = Vec::new();
        for _ in 0..2 {
            let mut conn = socket::TLSConnectionManager::new(
                "api.minecraftservices.com".to_string(),
                443,
                "api.minecraftservices.com".to_string(),
            );
            conn.connect(1);
            conn_vec.push(conn);
        }
        println!("Signed in to {}.", self.config.account.username);
        println!("Setup complete!");
        spin_sleep::sleep(time::Duration::from_secs(
            (droptime_epoch - Utc::now().timestamp() - offset as i64) as u64,
        ));
        // Snipe
        // Change skin if successful
    }
}
