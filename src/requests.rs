// My error handling is terrible :(


use crate::cli::pretty_panic;
use crate::{cli, config, constants};
use chrono::{DateTime, Duration, TimeZone, Utc};
use reqwest::{Body, Client};
use serde_json::{json, Value};
use std::{thread, time};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

pub struct Requests {
    client: Client,
}

impl Requests {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .use_rustls_tls()
                .build()
                .unwrap(),
        }
    }

    pub async fn authenticate_mojang(&self, username: &str, password: &str) -> String {
        let post_json = json!({
            "username": username,
            "password": password
        });
        let url = format!("{}/authenticate", constants::YGGDRASIL_ORIGIN_SERVER);
        let res = self
            .client
            .post(url)
            .json(&post_json)
            .header(reqwest::header::USER_AGENT, constants::USER_AGENT)
            .send()
            .await;
        let res = match res {
            Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
            _ => res.unwrap(),
        };
        match res.status().as_u16() {
            200 => {
                let v: Value = serde_json::from_str(&res.text().await.unwrap()).unwrap();
                let access_token = v["accessToken"].as_str().unwrap().to_string();
                access_token
            },
            403 => pretty_panic("Authentication error. Check if you have entered your username and password correctly."),
            code => pretty_panic(&format!("HTTP status code: {}", code)),
        }
    }

    pub async fn authenticate_microsoft(
        &self,
        username: &str,
        password: &str,
    ) -> (String, DateTime<Utc>) {
        fn oauth2_authentication() -> (String, DateTime<Utc>) {
            let url = constants::MS_AUTH_SERVER;
            println!("Opening browser...");
            thread::sleep(time::Duration::from_secs(3));
            let auth_time = Utc::now();
            if webbrowser::open(url).is_err() {
                println!("Seems like you are running this program in a headless environment. Copy the following URL into your browser:");
                println!("{}", constants::MS_AUTH_SERVER);
            }
            bunt::println!("{$red}Note: If you signed in with another Microsoft account recently and are experiencing auto sign-in behaviour, disable cookies on your browser.{/$}");
            let access_token = cli::get_access_token();
            (access_token, auth_time)
        }
        if !(username.is_empty() || password.is_empty()) {
            let post_json = json!({
                "username": username,
                "password": password
            });
            let url = format!("{}/simpleauth", constants::BUCKSHOT_API_SERVER);
            let auth_time = Utc::now();
            let res = self.client.post(url).json(&post_json).send().await;
            let res = match res {
                Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
                _ => res.unwrap(),
            };
            let status = res.status().as_u16();
            if status == 200 {
                let body = res.text().await.unwrap();
                let v: Value = serde_json::from_str(&body).unwrap();
                let access_token = v["access_token"].as_str().unwrap().to_string();
                (access_token, auth_time)
            } else {
                if status == 400 {
                    let body = res.text().await.unwrap();
                    let v: Value = serde_json::from_str(&body).unwrap();
                    let error_msg = v["error"].as_str().unwrap();
                    bunt::eprintln!("{$red}Error{/$}: SimpleAuth failed.");
                    eprintln!("Reason: {}", error_msg);
                } else {
                    bunt::eprintln!("{$red}Error{/$}: SimpleAuth failed.");
                    eprintln!("Reason: Unknown server error.");
                }
                println!("Reverting to OAuth2 authentication...");
                oauth2_authentication()
            }
        } else {
            oauth2_authentication()
        }
    }

    pub async fn get_sq_id(&self, access_token: &str) -> Option<[i64; 3]> {
        let url = format!("{}/user/security/challenges", constants::MOJANG_API_SERVER);
        let res = self.client.get(url).bearer_auth(access_token).send().await;
        let res = match res {
            Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
            _ => res.unwrap(),
        };
        if res.status().as_u16() != 200 {
            pretty_panic(&format!("HTTP status code: {}", res.status().as_u16()));
        }
        let body = res.text().await.unwrap();
        if body == "[]" {
            None
        } else {
            let v: Value = serde_json::from_str(&body).unwrap();
            let first = v[0]["answer"]["id"].as_i64().unwrap();
            let second = v[1]["answer"]["id"].as_i64().unwrap();
            let third = v[2]["answer"]["id"].as_i64().unwrap();
            Some([first, second, third])
        }
    }

    pub async fn send_sq(&self, access_token: &str, id: &[i64; 3], answer: &[&String; 3]) {
        if answer[0].is_empty() || answer[1].is_empty() || answer[2].is_empty() {
            pretty_panic(
                "Your account has security questions and you did not provide any answers.",
            );
        }
        let post_body = json!([
            {
                "id": id[0],
                "answer": answer[0],
            },
            {
                "id": id[1],
                "answer": answer[1],
            },
            {
                "id": id[2],
                "answer": answer[2]
            }
        ]);
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .json(&post_body)
            .send()
            .await;
        let res = match res {
            Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
            _ => res.unwrap(),
        };
        match res.status().as_u16() {
            204 => (),
            403 => pretty_panic("Authentication error. Check if you have entered your security questions correctly."),
            code => pretty_panic(&format!("HTTP status code: {}", code)),
        }
    }

    pub async fn check_name_availability_time(
        &self,
        username_to_snipe: &str,
        auth_time: Option<DateTime<Utc>>,
    ) -> DateTime<Utc> {
        let url = format!(
            "{}/droptime/{}",
            constants::TEUN_NAMEMC_API,
            username_to_snipe
        );
        let res = self.client.get(url).send().await;
        let res = match res {
            Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
            _ => res.unwrap(),
        };
        match res.status().as_u16() {
            200 => {
                let body = res.text().await.unwrap();
                let v: Value = serde_json::from_str(&body).unwrap();
                let epoch = v["UNIX"].as_i64().unwrap();
                let droptime = Utc.timestamp(epoch, 0);
                if let Some(auth) = auth_time {
                    if droptime - auth.to_owned() > Duration::days(1) {
                        pretty_panic("You cannot snipe a name available more than one day later if you are using a Microsoft account.");
                    }
                }
                droptime
            }
            _ => pretty_panic("This name is not dropping or has already dropped."),
        }
    }

    pub async fn get_searches(&self, username_to_snipe: &str) {
        let url = format!(
            "{}/searches/{}",
            constants::TEUN_NAMEMC_API,
            username_to_snipe
        );
        let res = self.client.get(url).send().await;
        match res {
            Err(e) if e.is_timeout() => {
                bunt::eprintln!("{$red}Error{/$}: HTTP request timeout.");
            }
            Err(_) => {}
            Ok(res) => {
                let body = res.text().await.unwrap();
                let v: Value = serde_json::from_str(&body).unwrap();
                match v["searches"].as_f64() {
                    Some(x) => {
                        bunt::println!(
                            "{$green}Successfully sniped {} with {} searches!{/$}",
                            username_to_snipe,
                            x
                        );
                    }
                    None => {
                        bunt::println!("{$green}Successfully sniped {}!{/$}", username_to_snipe);
                        bunt::eprintln!("{$red}Error{/$}: Failed to get number of name searches.");
                    }
                }
            }
        }
    }

    pub async fn check_name_change_eligibility(&self, access_token: &str) {
        let url = format!(
            "{}/minecraft/profile/namechange",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self.client.get(url).bearer_auth(access_token).send().await;
        let res = match res {
            Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
            _ => res.unwrap(),
        };
        if res.status().as_u16() != 200 {
            pretty_panic(&format!("HTTP status code: {}", res.status().as_u16()));
        }
        let body = res.text().await.unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        let is_allowed = v["nameChangeAllowed"].as_bool().unwrap();
        if !is_allowed {
            pretty_panic("You cannot name change within the cooldown period.")
        }
    }

    pub async fn upload_skin(&self, config: &config::Config, access_token: &str) {
        let img_file = match File::open(&config.config.skin_filename).await {
            Ok(f) => f,
            Err(_) => {
                bunt::eprintln!(
                    "{$red}Error{/$}: File {} not found.",
                    config.config.skin_filename
                );
                return;
            }
        };
        let stream = FramedRead::new(img_file, BytesCodec::new());
        let stream = Body::wrap_stream(stream);
        let image_part = reqwest::multipart::Part::stream(stream);
        let form = reqwest::multipart::Form::new()
            .text("variant", config.config.skin_model.to_owned())
            .part("file", image_part);
        let url = format!(
            "{}/minecraft/profile/skins",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .multipart(form)
            .send()
            .await;
        match res {
            Err(e) if e.is_timeout() => {
                bunt::eprintln!("{$red}Error{/$}: HTTP request timeout.");
            }
            Ok(res) => {
                if res.status().as_u16() == 200 {
                    bunt::println!("{$green}Successfully changed skin!{/$}")
                } else {
                    bunt::eprintln!("{$red}Error{/$}: Failed to upload skin.")
                }
            }
            Err(_) => {}
        };
    }

    pub async fn redeem_giftcode(&self, giftcode: &str, access_token: &str) {
        let url = format!(
            "{}/productvoucher/{}",
            constants::MINECRAFTSERVICES_API_SERVER,
            giftcode
        );
        let res = self
            .client
            .put(url)
            .bearer_auth(access_token)
            .json("")
            .send()
            .await;
        let res = match res {
            Err(e) if e.is_timeout() => pretty_panic("HTTP request timeout."),
            _ => res.unwrap(),
        };
        if res.status().as_u16() != 200 {
            pretty_panic(&format!("HTTP status code: {}", res.status().as_u16()));
        }
    }
}
