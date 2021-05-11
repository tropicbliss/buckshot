use crate::constants::SERVER_RESPONSE_DURATION;
use chrono::{DateTime, Local, Utc};
use native_tls;
use native_tls::TlsConnector;
use serde_json::json;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::{task, time};

pub async fn snipe_regular(
    snipe_time: DateTime<Utc>,
    username_to_snipe: &str,
    access_token: &str,
) -> bool {
    let mut handle_vec: Vec<task::JoinHandle<u16>> = Vec::new();
    let mut status_vec: Vec<u16> = Vec::new();
    for _ in 0..3 {
        let handle = task::spawn(async {
            snipe_task_regular(snipe_time, username_to_snipe, access_token).await
        });
        handle_vec.push(handle);
    }
    for handle in handle_vec {
        status_vec.push(handle.await.unwrap());
    }
    status_vec.contains(&200)
}

async fn snipe_task_regular(
    snipe_time: DateTime<Utc>,
    username_to_snipe: &str,
    access_token: &str,
) -> u16 {
    let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n", username_to_snipe).as_bytes();
    let data2 = format!("\r\n").as_bytes();
    let stream = TcpStream::connect("api.minecraftservices.com:443")
        .await
        .unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let mut stream = connector
        .connect("api.minecraftservices.com", stream)
        .await
        .unwrap();
    let mut res = Vec::new();
    stream.write_all(data).await.unwrap();
    time::sleep(
        snipe_time
            .signed_duration_since(Utc::now())
            .to_std()
            .unwrap(),
    )
    .await;
    stream.write_all(data2).await.unwrap();
    stream.read_to_end(&mut res).await.unwrap();
    let formatted_resp_time = Local::now().format("%F %T%.6f");
    let response = String::from_utf8_lossy(&res);
    let status = response[9..12].parse::<u16>().unwrap();
    let result = if status == 200 { "success" } else { "fail" };
    println!("[{}] {} @ {}", result, status, formatted_resp_time);
    status
}

pub async fn snipe_gc(
    snipe_time: DateTime<Utc>,
    username_to_snipe: &str,
    access_token: &str,
) -> bool {
    let mut handle_vec: Vec<task::JoinHandle<u16>> = Vec::new();
    let mut status_vec: Vec<u16> = Vec::new();
    for _ in 0..6 {
        let handle =
            task::spawn(async { snipe_task_gc(snipe_time, username_to_snipe, access_token).await });
        handle_vec.push(handle);
    }
    for handle in handle_vec {
        status_vec.push(handle.await.unwrap());
    }
    status_vec.contains(&200)
}

async fn snipe_task_gc(
    snipe_time: DateTime<Utc>,
    username_to_snipe: &str,
    access_token: &str,
) -> u16 {
    let payload = json!({ "profileName": username_to_snipe }).to_string();
    let data = format!("POST /minecraft/profile/ HTTP/1.1\r\nContent-Type: application/json\r\nHost: api.minecraftservices.com\r\nContent-Length: {}\r\nAuthorization: Bearer {}\r\n", payload.len(), access_token).as_bytes();
    let data2 = format!("\r\n{}", payload).as_bytes();
    let stream = TcpStream::connect("api.minecraftservices.com:443")
        .await
        .unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let mut stream = connector
        .connect("api.minecraftservices.com", stream)
        .await
        .unwrap();
    let mut res = Vec::new();
    stream.write_all(data).await.unwrap();
    time::sleep(
        snipe_time
            .signed_duration_since(Utc::now())
            .to_std()
            .unwrap(),
    )
    .await;
    stream.write_all(data2).await.unwrap();
    stream.read_to_end(&mut res).await.unwrap();
    let formatted_resp_time = Local::now().format("%F %T%.6f");
    let response = String::from_utf8_lossy(&res);
    let status = response[9..12].parse::<u16>().unwrap();
    let result = if status == 200 { "success" } else { "fail" };
    println!("[{}] {} @ {}", result, status, formatted_resp_time);
    status
}

pub async fn auto_offset_calculation_regular(username_to_snipe: &str) -> i32 {
    let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n", username_to_snipe).as_bytes();
    let data2 = format!("\r\n").as_bytes();
    let stream = TcpStream::connect("api.minecraftservices.com:443")
        .await
        .unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let mut stream = connector
        .connect("api.minecraftservices.com", stream)
        .await
        .unwrap();
    let mut res = Vec::new();
    stream.write_all(data).await.unwrap();
    let before = Instant::now();
    stream.write_all(data2).await.unwrap();
    stream.read_to_end(&mut res).await.unwrap();
    let after = Instant::now();
    after.checked_duration_since(before).unwrap().as_millis() as i32 - SERVER_RESPONSE_DURATION
}

pub async fn auto_offset_calculation_gc(username_to_snipe: &str) -> i32 {
    let payload = json!({ "profileName": username_to_snipe }).to_string();
    let data = format!("POST /minecraft/profile/ HTTP/1.1\r\nContent-Type: application/json\r\nHost: api.minecraftservices.com\r\nContent-Length: {}\r\nAuthorization: Bearer token\r\n", payload.len()).as_bytes();
    let data2 = format!("\r\n{}", payload).as_bytes();
    let stream = TcpStream::connect("api.minecraftservices.com:443")
        .await
        .unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let mut stream = connector
        .connect("api.minecraftservices.com", stream)
        .await
        .unwrap();
    let mut res = Vec::new();
    stream.write_all(data).await.unwrap();
    let before = Instant::now();
    stream.write_all(data2).await.unwrap();
    stream.read_to_end(&mut res).await.unwrap();
    let after = Instant::now();
    after.checked_duration_since(before).unwrap().as_millis() as i32 - SERVER_RESPONSE_DURATION
}
