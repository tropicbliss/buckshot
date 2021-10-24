use anyhow::Result;
use chrono::{DateTime, Duration, Local};
use native_tls::TlsConnector;
use serde_json::json;
use std::{net::ToSocketAddrs, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Barrier,
    task::JoinHandle,
    time::sleep,
};

pub struct ResData {
    pub status: u16,
    pub timestamp: DateTime<Local>,
    pub account_idx: usize,
}

pub async fn snipe_executor(
    name: &str,
    bearer_tokens: &[String],
    snipe_time: DateTime<Local>,
    is_gc: bool,
) -> Result<Vec<ResData>> {
    let req_count = if is_gc { 6 } else { 3 };
    let addr = "api.minecraftservices.com:443"
        .to_socket_addrs()?
        .next()
        .unwrap();
    let cx = TlsConnector::builder().build()?;
    let cx = tokio_native_tls::TlsConnector::from(cx);
    let cx = Arc::new(cx);
    let mut handles = Vec::with_capacity(req_count * bearer_tokens.len());
    let barrier = Arc::new(Barrier::new(req_count * bearer_tokens.len()));
    for (account_idx, bearer_token) in bearer_tokens.iter().enumerate() {
        let payload = if is_gc {
            let post_body = json!({ "profileName": name }).to_string();
            format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", bearer_token, post_body.len(), post_body).into_bytes()
        } else {
            format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer {}\r\n", name, bearer_token).into_bytes()
        };
        let payload = Arc::new(payload);
        for _ in 0..req_count {
            let cx = Arc::clone(&cx);
            let payload = Arc::clone(&payload);
            let c = barrier.clone();
            let mut buf = [0; 12];
            let handshake_time = snipe_time - Duration::seconds(32);
            let handle: JoinHandle<Result<_, anyhow::Error>> = tokio::task::spawn(async move {
                let sleep_duration = (handshake_time - Local::now())
                    .to_std()
                    .unwrap_or(std::time::Duration::ZERO);
                sleep(sleep_duration).await;
                let socket = TcpStream::connect(&addr).await?;
                let mut socket = cx.connect("api.minecraftservices.com", socket).await?;
                socket.write_all(&payload).await?;
                let sleep_duration = (snipe_time - Local::now())
                    .to_std()
                    .unwrap_or(std::time::Duration::ZERO);
                sleep(sleep_duration).await;
                socket.write_all(b"\r\n").await?;
                c.wait().await;
                socket.read_exact(&mut buf).await?;
                let timestamp = Local::now();
                let res = String::from_utf8_lossy(&buf[..]);
                let status: u16 = res[9..].parse()?;
                let res_data = ResData {
                    status,
                    timestamp,
                    account_idx,
                };
                Ok(res_data)
            });
            handles.push(handle);
        }
    }
    let mut res_vec = Vec::with_capacity(req_count * bearer_tokens.len());
    for handle in handles {
        let res_data = handle.await??;
        res_vec.push(res_data);
    }
    res_vec.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(res_vec)
}
