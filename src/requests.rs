use crate::cli::pretty_panic;
use crate::{cli, config, constants};
use chrono::{DateTime, Duration, TimeZone, Utc};
use native_tls::TlsConnector;
use reqwest::Client;
use serde_json::{json, Value};
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::net::ToSocketAddrs;
use std::{thread, time};
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use webbrowser;

#[derive(Clone)]
pub struct Requests {
    client: Client,
}

impl Requests {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn authenticate_mojang(
        &self,
        username: &str,
        password: &str,
    ) -> (String, Option<DateTime<Utc>>) {
        let post_json = json!({
            "agent": {
                "name": "Minecraft",
                "version": 1
            },
            "username": username,
            "password": password,
            "clientToken": "Mojang-API-Client",
            "requestUser": "true"
        });
        let url = format!("{}/authenticate", constants::YGGDRASIL_ORIGIN_SERVER);
        let res = self.client.post(url).json(&post_json).send().await.unwrap();
        match res.status().as_u16() {
            200 => {
                let v: Value = serde_json::from_str(&res.text().await.unwrap()).unwrap();
                let access_token = v["accessToken"].as_str().unwrap().to_string();
                (access_token, None)
            },
            403 => pretty_panic("Authentication error. Check if you have entered your username and password correctly."),
            code => pretty_panic(&format!("HTTP status code: {}", code)),
        }
    }

    pub fn authenticate_microsoft(&self) -> (String, Option<DateTime<Utc>>) {
        let url = constants::MS_AUTH_SERVER;
        println!("Opening browser...");
        thread::sleep(time::Duration::from_secs(3));
        let auth_time = Utc::now();
        if webbrowser::open(url).is_err() {
            println!("Looks like you are running this program in a headless environment. Copy the following URL into your browser:");
            println!("{}", constants::MS_AUTH_SERVER);
        }
        let access_token = cli::get_access_token();
        (access_token, Some(auth_time))
    }

    pub async fn check_sq(&self, access_token: &str) -> bool {
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();
        match res.status().as_u16() {
            204 => false,
            403 => true,
            code => pretty_panic(&format!("HTTP status code: {}", code)),
        }
    }

    pub async fn get_sq_id(&self, access_token: &str) -> Option<[u8; 3]> {
        let url = format!("{}/user/security/challenges", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            pretty_panic(&format!("HTTP status code: {}", res.status().as_u16()));
        }
        let body = res.text().await.unwrap();
        if body == "[]" {
            None
        } else {
            let v: Value = serde_json::from_str(&body).unwrap();
            let first = v[0]["answer"]["id"].as_u64().unwrap() as u8;
            let second = v[1]["answer"]["id"].as_u64().unwrap() as u8;
            let third = v[2]["answer"]["id"].as_u64().unwrap() as u8;
            Some([first, second, third])
        }
    }

    pub async fn send_sq(&self, access_token: &str, id: [u8; 3], answer: [&String; 3]) {
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
        ])
        .to_string();
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .body(post_body)
            .send()
            .await
            .unwrap();
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
        let res = self.client.get(url).send().await.unwrap();
        match res.status().as_u16() {
            200 => (),
            400 => pretty_panic("This name has not been cached yet."),
            code => pretty_panic(&format!("HTTP status code: {}", code)),
        }
        let body = res.text().await.unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        let epoch = match v.get("UNIX") {
            Some(droptime) => droptime,
            None => pretty_panic("Error checking droptime. Check if username is freely available."),
        }
        .as_i64()
        .unwrap();
        let droptime = Utc.timestamp(epoch, 0);
        if let Some(auth) = auth_time {
            if droptime.signed_duration_since(auth) > Duration::days(1) {
                pretty_panic("You cannot snipe a name available more than one day later if you are using a Microsoft account.");
            }
        }
        droptime
    }

    pub async fn check_name_change_eligibility(&self, access_token: &str) {
        let url = format!(
            "{}/minecraft/profile/namechange",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
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
        let img_byte = match File::open(&config.config.skin_filename) {
            Ok(f) => {
                let mut v: Vec<u8> = Vec::new();
                let mut br = BufReader::new(f);
                br.read_to_end(&mut v).unwrap();
                v
            }
            Err(_) => pretty_panic(&format!("File {} not found.", config.config.skin_filename)),
        };
        let image_part = reqwest::multipart::Part::bytes(img_byte);
        let form = reqwest::multipart::Form::new()
            .text("variant", config.config.skin_model.clone())
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
            .await
            .unwrap();
        if res.status().is_success() {
            bunt::println!("{$green}Successfully changed skin!{/$}")
        } else {
            bunt::eprintln!("{$red}Error{/$}: Failed to upload skin.")
        }
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
            .await
            .unwrap();
        if !res.status().is_success() {
            pretty_panic(&format!("HTTP status code: {}", res.status().as_u16()));
        }
    }
}

pub async fn auto_offset_calculation_regular(username_to_snipe: &str) -> i32 {
    println!("Measuring offset...");
    let addr = constants::MINECRAFTSERVICES_API_SERVER
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n\r\n", username_to_snipe).as_bytes();
    let mut stream = connector
        .connect(constants::MINECRAFTSERVICES_API_SERVER, stream)
        .await
        .unwrap();
    let before = time::Instant::now();
    stream.write_all(data).await.unwrap();
    let after = time::Instant::now();
    let offset = (after - before).as_millis() as i32;
    println!("Your offset is: {} ms.", offset);
    offset
}

pub async fn auto_offset_calculation_gc(username_to_snipe: &str) -> i32 {
    println!("Measuring offset...");
    let addr = constants::MINECRAFTSERVICES_API_SERVER
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let post_body = json!({ "profileName": username_to_snipe }).to_string();
    let data = format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n\r\n{}\r\n", post_body).as_bytes();
    let mut stream = connector
        .connect(constants::MINECRAFTSERVICES_API_SERVER, stream)
        .await
        .unwrap();
    let before = time::Instant::now();
    stream.write_all(data).await.unwrap();
    let after = time::Instant::now();
    let offset = (after - before).as_millis() as i32;
    println!("Your offset is: {} ms.", offset);
    offset
}

pub async fn snipe_gc(
    snipe_time: DateTime<Utc>,
    username_to_snipe: String,
    access_token: String,
    spread_offset: i32,
) -> bool {
    let mut handle_vec = Vec::new();
    let mut status_vec = Vec::new();
    let mut spread = 0;
    for _ in 0..constants::GC_SNIPE_REQS {
        let access_token = access_token.clone();
        let username_to_snipe = username_to_snipe.clone();
        let handle = tokio::task::spawn(async move {
            let snipe_time = snipe_time + Duration::milliseconds(spread);
            let handshake_time = snipe_time - Duration::seconds(20);
            let mut res = Vec::new();
            let addr = constants::MINECRAFTSERVICES_API_SERVER
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap();
            let connector = TlsConnector::builder().build().unwrap();
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let post_body = json!({ "profileName": username_to_snipe }).to_string();
            let data = format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n\r\n{}\r\n", post_body, access_token).as_bytes();
            tokio::time::sleep((handshake_time - Utc::now()).to_std().unwrap()).await;
            let stream = TcpStream::connect(&addr).await.unwrap();
            let mut stream = connector
                .connect(constants::MINECRAFTSERVICES_API_SERVER, stream)
                .await
                .unwrap();
            tokio::time::sleep((snipe_time - Utc::now()).to_std().unwrap()).await;
            stream.write_all(data).await.unwrap();
            stream.read_to_end(&mut res).await.unwrap();
            let formatted_resp_time = Utc::now().format("%F %T%.6f");
            let response = String::from_utf8_lossy(&res);
            let status = response[9..12].parse::<u16>().unwrap();
            if status == 200 {
                bunt::println!(
                    "[{$green}success{/$}] {$green}200{/$} @ {[cyan]}",
                    formatted_resp_time
                )
            } else {
                bunt::println!(
                    "[{$red}fail{/$}] {[red]} @ {[cyan]}",
                    status,
                    formatted_resp_time
                )
            }
            status
        });
        handle_vec.push(handle);
        spread += spread_offset as i64;
    }
    for handle in handle_vec {
        status_vec.push(handle.await.unwrap());
    }
    status_vec.contains(&200)
}

pub async fn snipe_regular(
    snipe_time: DateTime<Utc>,
    username_to_snipe: String,
    access_token: String,
    spread_offset: i32,
) -> bool {
    let mut handle_vec = Vec::new();
    let mut status_vec = Vec::new();
    let mut spread = 0;
    for _ in 0..constants::REGULAR_SNIPE_REQS {
        let access_token = access_token.clone();
        let username_to_snipe = username_to_snipe.clone();
        let handle = tokio::task::spawn(async move {
            let snipe_time = snipe_time + Duration::milliseconds(spread);
            let handshake_time = snipe_time - Duration::seconds(20);
            let mut res = Vec::new();
            let addr = constants::MINECRAFTSERVICES_API_SERVER
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap();
            let connector = TlsConnector::builder().build().unwrap();
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n\r\n", username_to_snipe, access_token).as_bytes();
            tokio::time::sleep((handshake_time - Utc::now()).to_std().unwrap()).await;
            let stream = TcpStream::connect(&addr).await.unwrap();
            let mut stream = connector
                .connect(constants::MINECRAFTSERVICES_API_SERVER, stream)
                .await
                .unwrap();
            tokio::time::sleep((snipe_time - Utc::now()).to_std().unwrap()).await;
            stream.write_all(data).await.unwrap();
            stream.read_to_end(&mut res).await.unwrap();
            let formatted_resp_time = Utc::now().format("%F %T%.6f");
            let response = String::from_utf8_lossy(&res);
            let status = response[9..12].parse::<u16>().unwrap();
            if status == 200 {
                bunt::println!(
                    "[{$green}success{/$}] {$green}200{/$} @ {[cyan]}",
                    formatted_resp_time
                )
            } else {
                bunt::println!(
                    "[{$red}fail{/$}] {[red]} @ {[cyan]}",
                    status,
                    formatted_resp_time
                )
            }
            status
        });
        handle_vec.push(handle);
        spread += spread_offset as i64;
    }
    for handle in handle_vec {
        status_vec.push(handle.await.unwrap());
    }
    status_vec.contains(&200)
}
