use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use native_tls::TlsConnector;
use serde_json::json;
use std::{convert::TryFrom, net::ToSocketAddrs, sync::Arc, time::Instant};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Barrier,
    task::JoinHandle,
    time::sleep,
};

pub struct Executor<'a> {
    name: &'a str,
}

pub struct ResData {
    pub status: u16,
    pub timestamp: DateTime<Utc>,
    pub account_idx: usize,
}

impl<'a> Executor<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }

    pub async fn auto_offset_calculator(&self) -> Result<i64> {
        const SERVER_RES_TIME: i64 = 40;
        let mut buf = [0; 12];
        let addr = "api.minecraftservices.com:443"
            .to_socket_addrs()?
            .next()
            .unwrap();
        let payload = format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer token\r\n", self.name).into_bytes();
        let socket = TcpStream::connect(&addr).await?;
        let cx = TlsConnector::builder().build()?;
        let cx = tokio_native_tls::TlsConnector::from(cx);
        let mut socket = cx.connect("api.minecraftservices.com", socket).await?;
        socket.write_all(&payload).await?;
        let start = Instant::now();
        socket.write_all(b"\r\n").await?;
        socket.read_exact(&mut buf).await?;
        let elapsed = start.elapsed();
        Ok((i64::try_from((elapsed).as_millis())? - SERVER_RES_TIME) / 2)
    }

    pub async fn snipe_executor(
        &self,
        bearer_tokens: &[String],
        spread_offset: usize,
        snipe_time: DateTime<Utc>,
        is_gc: bool,
    ) -> Result<Vec<ResData>> {
        let req_count = if is_gc { 6 } else { 3 };
        let mut spread = 0;
        let addr = "api.minecraftservices.com:443"
            .to_socket_addrs()?
            .next()
            .unwrap();
        let cx = TlsConnector::builder().build()?;
        let cx = tokio_native_tls::TlsConnector::from(cx);
        let cx = Arc::new(cx);
        let mut handle_vec = Vec::with_capacity(req_count * bearer_tokens.len());
        let barrier = if spread_offset != 0 {
            Arc::new(Barrier::new(0))
        } else {
            Arc::new(Barrier::new(req_count * bearer_tokens.len()))
        };
        for (account_idx, bearer_token) in bearer_tokens.iter().enumerate() {
            let payload = if is_gc {
                let post_body = json!({ "profileName": self.name }).to_string();
                format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", bearer_token, post_body.len(), post_body).into_bytes()
            } else {
                format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer {}\r\n", self.name, bearer_token).into_bytes()
            };
            let payload = Arc::new(payload);
            for _ in 0..req_count {
                let cx = Arc::clone(&cx);
                let payload = Arc::clone(&payload);
                let c = barrier.clone();
                let mut buf = [0; 12];
                let snipe_time = snipe_time + Duration::milliseconds(spread);
                let handshake_time = snipe_time - Duration::seconds(32);
                let handle: JoinHandle<Result<_, anyhow::Error>> = tokio::task::spawn(async move {
                    let sleep_duration = (handshake_time - Utc::now())
                        .to_std()
                        .unwrap_or(std::time::Duration::ZERO);
                    sleep(sleep_duration).await;
                    let socket = TcpStream::connect(&addr).await?;
                    let mut socket = cx.connect("api.minecraftservices.com", socket).await?;
                    socket.write_all(&payload).await?;
                    let sleep_duration = (snipe_time - Utc::now())
                        .to_std()
                        .unwrap_or(std::time::Duration::ZERO);
                    sleep(sleep_duration).await;
                    socket.write_all(b"\r\n").await?;
                    c.wait().await;
                    socket.read_exact(&mut buf).await?;
                    let timestamp = Utc::now();
                    let res = String::from_utf8_lossy(&buf[..]);
                    let status: u16 = res[9..].parse()?;
                    let res_data = ResData {
                        status,
                        timestamp,
                        account_idx,
                    };
                    Ok(res_data)
                });
                spread += spread_offset as i64;
                handle_vec.push(handle);
            }
        }
        let mut res_vec = Vec::with_capacity(req_count * bearer_tokens.len());
        for handle in handle_vec {
            let res_data = handle.await??;
            res_vec.push(res_data);
        }
        res_vec.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(res_vec)
    }
}
