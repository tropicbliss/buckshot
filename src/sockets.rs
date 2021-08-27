use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use console::style;
use native_tls::TlsConnector;
use serde_json::json;
use std::convert::TryFrom;
use std::io::{stdout, Write};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::sleep;

pub struct Executor<'a> {
    name: &'a str,
    is_gc: bool,
}

impl<'a> Executor<'a> {
    pub fn new(name: &'a str, is_gc: bool) -> Self {
        Self { name, is_gc }
    }

    pub async fn auto_offset_calculator(&self) -> Result<i64> {
        const SERVER_RES_TIME: i64 = 40;
        let mut buf = [0; 12];
        let addr = "api.minecraftservices.com:443"
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Invalid socket address"))?;
        let payload = if self.is_gc {
            let post_body = json!({ "profileName": self.name }).to_string();
            format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer token\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", post_body.len(), post_body).into_bytes()
        } else {
            format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer token\r\n", self.name).into_bytes()
        };
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
        bearer_token: &str,
        spread_offset: usize,
        snipe_time: DateTime<Utc>,
    ) -> Result<bool> {
        let mut is_success = false;
        let req_count = if self.is_gc { 6 } else { 3 };
        let mut spread = 0;
        let payload = if self.is_gc {
            let post_body = json!({ "profileName": self.name }).to_string();
            format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", bearer_token, post_body.len(), post_body).into_bytes()
        } else {
            format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nConnection: close\r\nAuthorization: Bearer {}\r\n", self.name, bearer_token).into_bytes()
        };
        let payload = Arc::new(payload);
        let addr = "api.minecraftservices.com:443"
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Invalid socket address"))?;
        let cx = TlsConnector::builder().build()?;
        let cx = tokio_native_tls::TlsConnector::from(cx);
        let cx = Arc::new(cx);
        let handle_vec: Vec<JoinHandle<Result<_, anyhow::Error>>> = (0..req_count)
            .map(|_| {
                let cx = Arc::clone(&cx);
                let payload = Arc::clone(&payload);
                let handle = tokio::task::spawn(async move {
                    let mut buf = [0; 12];
                    let snipe_time = snipe_time + Duration::milliseconds(spread);
                    let handshake_time = snipe_time - Duration::seconds(3);
                    let sleep_duration = match (handshake_time - Utc::now()).to_std() {
                        Ok(x) => x,
                        Err(_) => std::time::Duration::ZERO,
                    };
                    sleep(sleep_duration).await;
                    let socket = TcpStream::connect(&addr).await?;
                    let mut socket = cx.connect("api.minecraftservices.com", socket).await?;
                    socket.write_all(&payload).await?;
                    let sleep_duration = match (handshake_time - Utc::now()).to_std() {
                        Ok(x) => x,
                        Err(_) => std::time::Duration::ZERO,
                    };
                    sleep(sleep_duration).await;
                    socket.write_all(b"\r\n").await?;
                    socket.read_exact(&mut buf).await?;
                    let formatted_res_time = Utc::now().format("%F %T%.6f");
                    let res = String::from_utf8_lossy(&buf[..]);
                    let status: u16 = res[9..].parse()?;
                    match status {
                        200 => {
                            writeln!(
                                stdout(),
                                "[{}] {} @ {}",
                                style("success").green(),
                                style("200").green(),
                                style(format!("{}", formatted_res_time)).cyan()
                            )?;
                            Ok(true)
                        }
                        status => {
                            writeln!(
                                stdout(),
                                "[{}] {} @ {}",
                                style("fail").red(),
                                style(format!("{}", status)).red(),
                                style(format!("{}", formatted_res_time)).cyan()
                            )?;
                            Ok(false)
                        }
                    }
                });
                spread += spread_offset as i64;
                handle
            })
            .collect();
        for handle in handle_vec {
            let status = handle.await??;
            if status {
                is_success = true;
            }
        }
        Ok(is_success)
    }
}
