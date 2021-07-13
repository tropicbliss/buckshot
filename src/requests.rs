// My error handling is terrible :(

use crate::{cli, config, constants};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{Body, Client};
use serde_json::{json, Value};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

pub enum AuthenicationError {
    RetryableAuthenticationError,
}

pub enum NameAvailabilityError {
    NameNotAvailableError,
}

pub struct NameMC {
    pub droptime: DateTime<Utc>,
    pub searches: u32,
}

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
        let function_id = "YggdrasilAuth";
        if username.is_empty() || password.is_empty() {
            cli::pretty_panik(function_id, "You did not provide a username or password.");
        }
        let post_json = json!({
            "username": username,
            "password": password
        });
        let url = format!("{}/authenticate", constants::YGGDRASIL_ORIGIN_SERVER);
        let res = self
            .client
            .post(url)
            .json(&post_json)
            .header(reqwest::header::USER_AGENT, constants::AUTH_USER_AGENT)
            .send()
            .await;
        let res = match res {
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        match res.status().as_u16() {
            200 => {
                let v: Value = serde_json::from_str(&res.text().await.unwrap()).unwrap();
                let access_token = v["accessToken"].as_str().unwrap().to_string();
                access_token
            },
            403 => cli::pretty_panik(function_id, "Authentication error. Please check if you have entered your username and password correctly."),
            status => cli::http_not_ok_panik(function_id, status),
        }
    }

    pub async fn authenticate_microsoft(
        &self,
        username: &str,
        password: &str,
    ) -> Result<String, AuthenicationError> {
        let function_id = "MicroAuth";
        if username.is_empty() || password.is_empty() {
            cli::pretty_panik(function_id, "You did not provide a username or password.");
        }
        let post_json = json!({
            "username": username,
            "password": password
        });
        let url = format!("{}/simpleauth", constants::BUCKSHOT_API_SERVER);
        let res = self.client.post(url).json(&post_json).send().await;
        let res = match res {
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        match res.status().as_u16() {
            200 => {
                let body = res.text().await.unwrap();
                let v: Value = serde_json::from_str(&body).unwrap();
                Ok(v["access_token"].as_str().unwrap().to_string())
            }
            400 => {
                let body = res.text().await.unwrap();
                let v: Value = serde_json::from_str(&body).unwrap();
                let err = v["error"].as_str().unwrap().to_string();
                if err == "This API is currently overloaded. Please try again later." {
                    Err(AuthenicationError::RetryableAuthenticationError)
                } else {
                    cli::pretty_panik(
                        function_id,
                        &format!("Authentication error. Reason: {}", err),
                    )
                }
            }
            status => cli::http_not_ok_panik(function_id, status),
        }
    }

    pub async fn get_sq_id(&self, access_token: &str) -> Option<[i64; 3]> {
        let function_id = "GetSQID";
        let url = format!("{}/user/security/challenges", constants::MOJANG_API_SERVER);
        let res = self.client.get(url).bearer_auth(access_token).send().await;
        let res = match res {
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        let status = res.status().as_u16();
        if status != 200 {
            cli::http_not_ok_panik(function_id, status);
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
        let function_id = "SendSQ";
        if answer[0].is_empty() || answer[1].is_empty() || answer[2].is_empty() {
            cli::pretty_panik(
                function_id,
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
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        match res.status().as_u16() {
            204 => (),
            403 => cli::pretty_panik(function_id, "Authentication error. Check if you have entered your security questions correctly."),
            status => cli::http_not_ok_panik(function_id, status),
        }
    }

    pub async fn check_name_availability_time(
        &self,
        username_to_snipe: &str,
    ) -> Result<NameMC, NameAvailabilityError> {
        let function_id = "GetDrop";
        let url = format!("{}/droptime?name={}", constants::NAMEMC_API, username_to_snipe);
        let res = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, constants::NAMEMC_USER_AGENT)
            .send()
            .await;
        let res = match res {
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        let status = res.status().as_u16();
        if status != 200 {
            cli::http_not_ok_panik(function_id, status);
        }
        let body = res.text().await.unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        match v.get("UNIX") {
            Some(x) => {
                let searches = v["searches"].as_i64().unwrap() as u32;
                let epoch = x.as_i64().unwrap();
                let droptime = Utc.timestamp(epoch, 0);
                Ok(NameMC { droptime, searches })
            }
            None => Err(NameAvailabilityError::NameNotAvailableError),
        }
    }

    pub async fn check_name_change_eligibility(&self, access_token: &str) {
        let function_id = "CheckEligible";
        let url = format!(
            "{}/minecraft/profile/namechange",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self.client.get(url).bearer_auth(access_token).send().await;
        let res = match res {
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        let status = res.status().as_u16();
        if status != 200 {
            cli::http_not_ok_panik(function_id, status);
        }
        let body = res.text().await.unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        let is_allowed = v["nameChangeAllowed"].as_bool().unwrap();
        if !is_allowed {
            cli::pretty_panik(
                function_id,
                "You cannot name change within the cooldown period.",
            )
        }
    }

    pub async fn upload_skin(&self, config: &config::Config, access_token: &str) {
        let function_id = "SkinUpload";
        let img_file = match File::open(&config.config.skin_filename).await {
            Ok(f) => f,
            Err(_) => {
                cli::kalm_panik(
                    function_id,
                    &format!("File {} not found.", config.config.skin_filename),
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
                cli::http_timeout_panik(function_id);
            }
            Ok(res) => match res.status().as_u16() {
                200 => bunt::println!("{$green}Successfully changed skin!{/$}"),
                status => cli::kalm_panik(
                    function_id,
                    &format!("Failed to change skin. HTTP status code: {}.", status),
                ),
            },
            Err(e) => panic!("{}", e),
        };
    }

    pub async fn redeem_giftcode(&self, giftcode: &str, access_token: &str) {
        let function_id = "GCRedeem";
        let url = format!(
            "{}/productvoucher/{}",
            constants::MINECRAFTSERVICES_API_SERVER,
            giftcode
        );
        let res = self
            .client
            .put(url)
            .bearer_auth(access_token)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await;
        let res = match res {
            Err(e) if e.is_timeout() => cli::http_timeout_panik(function_id),
            _ => res.unwrap(),
        };
        let status = res.status().as_u16();
        if status != 200 {
            cli::http_not_ok_panik(function_id, status);
        }
    }
}
