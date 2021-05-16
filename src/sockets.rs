use crate::constants;
use chrono::{DateTime, Duration, Utc};
use native_tls::TlsConnector;
use serde_json::json;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub async fn auto_offset_calculation_regular(username_to_snipe: &str) -> i32 {
    println!("Measuring offset...");
    let mut buf = [0; 12];
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or("failed to resolve api.minecraftservices.com")
        .unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nConnection: close\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n", username_to_snipe);
    let data = data.as_bytes();
    let mut stream = connector
        .connect("api.minecraftservices.com", stream)
        .await
        .unwrap();
    stream.write_all(data).await.unwrap();
    let before = Instant::now();
    stream.write_all(b"\r\n").await.unwrap();
    stream.read(&mut buf).await.unwrap();
    let after = Instant::now();
    let offset = ((after - before).as_millis() as i32 - constants::SERVER_RESPONSE_TIME as i32) / 2;
    println!("Your offset is: {} ms.", offset);
    offset
}

pub async fn auto_offset_calculation_gc(username_to_snipe: &str) -> i32 {
    println!("Measuring offset...");
    let mut buf = [0; 12];
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or("failed to resolve api.minecraftservices.com")
        .unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let connector = TlsConnector::builder().build().unwrap();
    let connector = tokio_native_tls::TlsConnector::from(connector);
    let post_body = json!({ "profileName": username_to_snipe }).to_string();
    let data = format!("POST /minecraft/profile HTTP/1.1\r\nConnection: close\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n\r\n{}", post_body);
    let data = data.as_bytes();
    let mut stream = connector
        .connect("api.minecraftservices.com", stream)
        .await
        .unwrap();
    stream.write_all(data).await.unwrap();
    let before = Instant::now();
    stream.write_all(b"\r\n").await.unwrap();
    stream.read(&mut buf).await.unwrap();
    let after = Instant::now();
    let offset = ((after - before).as_millis() as i32 - constants::SERVER_RESPONSE_TIME as i32) / 2;
    println!("Your offset is: {} ms.", offset);
    offset
}

pub async fn snipe_gc(
    snipe_time: DateTime<Utc>,
    username_to_snipe: String,
    access_token: String,
    spread_offset: i32,
) -> bool {
    let mut status_vec = Vec::new();
    let mut spread = 0;
    let access_token = Arc::new(access_token);
    let username_to_snipe = Arc::new(username_to_snipe);
    let (tx, mut rx) = mpsc::channel(constants::GC_SNIPE_REQS as usize);
    for _ in 0..constants::GC_SNIPE_REQS {
        let access_token = Arc::clone(&access_token);
        let username_to_snipe = Arc::clone(&username_to_snipe);
        let tx_cloned = tx.clone();
        tokio::task::spawn(async move {
            let snipe_time = snipe_time + Duration::milliseconds(spread);
            let handshake_time = snipe_time - Duration::seconds(5);
            let mut buf = [0; 12];
            let addr = "api.minecraftservices.com:443"
                .to_socket_addrs()
                .unwrap()
                .next()
                .ok_or("failed to resolve api.minecraftservices.com")
                .unwrap();
            let connector = TlsConnector::builder().build().unwrap();
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let post_body = json!({ "profileName": *username_to_snipe }).to_string();
            let data = format!("POST /minecraft/profile HTTP/1.1\r\nConnection: close\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n\r\n{}", post_body, access_token);
            let data = data.as_bytes();
            sleep((handshake_time - Utc::now()).to_std().unwrap()).await;
            let stream = TcpStream::connect(&addr).await.unwrap();
            let mut stream = connector
                .connect("api.minecraftservices.com", stream)
                .await
                .unwrap();
            stream.write_all(data).await.unwrap();
            sleep((snipe_time - Utc::now()).to_std().unwrap()).await;
            stream.write_all(b"\r\n").await.unwrap();
            stream.read(&mut buf).await.unwrap();
            let formatted_resp_time = Utc::now().format("%F %T%.6f");
            let res = String::from_utf8_lossy(&mut buf);
            let status = res[9..].parse::<u16>().unwrap();
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
            tx_cloned.send(status).await.unwrap();
        });
        spread += spread_offset as i64;
    }
    while let Some(status_code) = rx.recv().await {
        status_vec.push(status_code);
    }
    status_vec.contains(&200)
}

pub async fn snipe_regular(
    snipe_time: DateTime<Utc>,
    username_to_snipe: String,
    access_token: String,
    spread_offset: i32,
) -> bool {
    let mut status_vec = Vec::new();
    let mut spread = 0;
    let access_token = Arc::new(access_token);
    let username_to_snipe = Arc::new(username_to_snipe);
    let (tx, mut rx) = mpsc::channel(constants::REGULAR_SNIPE_REQS as usize);
    for _ in 0..constants::REGULAR_SNIPE_REQS {
        let access_token = Arc::clone(&access_token);
        let username_to_snipe = Arc::clone(&username_to_snipe);
        let tx_cloned = tx.clone();
        tokio::task::spawn(async move {
            let snipe_time = snipe_time + Duration::milliseconds(spread);
            let handshake_time = snipe_time - Duration::seconds(5);
            let mut buf = [0; 12];
            let addr = "api.minecraftservices.com:443"
                .to_socket_addrs()
                .unwrap()
                .next()
                .ok_or("failed to resolve api.minecraftservices.com")
                .unwrap();
            let connector = TlsConnector::builder().build().unwrap();
            let connector = tokio_native_tls::TlsConnector::from(connector);
            let data = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nConnection: close\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n", username_to_snipe, access_token);
            let data = data.as_bytes();
            sleep((handshake_time - Utc::now()).to_std().unwrap()).await;
            let stream = TcpStream::connect(&addr).await.unwrap();
            let mut stream = connector
                .connect("api.minecraftservices.com", stream)
                .await
                .unwrap();
            stream.write_all(data).await.unwrap();
            sleep((snipe_time - Utc::now()).to_std().unwrap()).await;
            stream.write_all(b"\r\n").await.unwrap();
            stream.read(&mut buf).await.unwrap();
            let formatted_resp_time = Utc::now().format("%F %T%.6f");
            let res = String::from_utf8_lossy(&mut buf);
            let status = res[9..].parse::<u16>().unwrap();
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
            tx_cloned.send(status).await.unwrap();
        });
        spread += spread_offset as i64;
    }
    while let Some(status_code) = rx.recv().await {
        status_vec.push(status_code);
    }
    status_vec.contains(&200)
}
