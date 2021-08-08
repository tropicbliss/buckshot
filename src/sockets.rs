use crate::constants;
use ansi_term::Colour::{Cyan, Green, Red};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
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
use tokio_rustls::{rustls::ClientConfig, webpki::DNSNameRef, TlsConnector};

pub struct Executor {
    name: String,
    is_gc: bool,
}

impl Executor {
    pub fn new(name: String, is_gc: bool) -> Self {
        Self { name, is_gc }
    }

    pub async fn auto_offset_calculator(&self) -> Result<i64> {
        let mut buf = [0; 12];
        let addr = "api.minecraftservices.com:443"
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Invalid SocketAddr used"))?;
        let payload = if self.is_gc {
            let post_body = json!({ "profileName": self.name }).to_string();
            format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n\r\n{}", post_body).into_bytes()
        } else {
            format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer token\r\n", self.name).into_bytes()
        };
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        let connector = TlsConnector::from(Arc::new(config));
        let domain = DNSNameRef::try_from_ascii_str("api.minecraftservices.com")?;
        let stream = TcpStream::connect(&addr).await?;
        let mut stream = connector.connect(domain, stream).await?;
        stream.write_all(&payload).await?;
        let before = Instant::now();
        stream.write_all(b"\r\n").await?;
        stream.read_exact(&mut buf).await?;
        let after = Instant::now();
        Ok((i64::try_from((after - before).as_millis())? - constants::SERVER_RESPONSE_TIME) / 2)
    }

    pub async fn snipe_executor(
        &self,
        access_token: &str,
        spread_offset: usize,
        snipe_time: DateTime<Utc>,
    ) -> Result<bool> {
        let payload = if self.is_gc {
            let post_body = json!({ "profileName": self.name }).to_string();
            format!("POST /minecraft/profile HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAccept: application/json\r\nAuthorization: Bearer {}\r\n\r\n{}", access_token, post_body).into_bytes()
        } else {
            format!("PUT /minecraft/profile/name/{} HTTP/1.1\r\nHost: api.minecraftservices.com\r\nAuthorization: Bearer {}\r\n", self.name, access_token).into_bytes()
        };
        let req_count = if self.is_gc {
            constants::GC_SNIPE_REQS
        } else {
            constants::REGULAR_SNIPE_REQS
        };
        let mut status_vec = Vec::with_capacity(req_count);
        let mut handle_vec: Vec<JoinHandle<Result<_, anyhow::Error>>> =
            Vec::with_capacity(req_count);
        let mut spread = 0;
        let addr = "api.minecraftservices.com:443"
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Invalid SocketAddr used"))?;
        let data = Arc::new(payload);
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        let connector = Arc::new(TlsConnector::from(Arc::new(config)));
        let domain = DNSNameRef::try_from_ascii_str("api.minecraftservices.com")?;
        for _ in 0..req_count {
            let connector = Arc::clone(&connector);
            let data = Arc::clone(&data);
            let handle = tokio::task::spawn(async move {
                let mut buf = [0; 12];
                let snipe_time = snipe_time + Duration::milliseconds(spread);
                let handshake_time = snipe_time - Duration::seconds(3);
                let sleep_duration = match (handshake_time - Utc::now()).to_std() {
                    Ok(x) => x,
                    Err(_) => std::time::Duration::ZERO,
                };
                sleep(sleep_duration).await;
                let stream = TcpStream::connect(&addr).await?;
                let mut stream = connector.connect(domain, stream).await?;
                stream.write_all(&data).await?;
                let sleep_duration = match (snipe_time - Utc::now()).to_std() {
                    Ok(x) => x,
                    Err(_) => std::time::Duration::ZERO,
                };
                sleep(sleep_duration).await;
                stream.write_all(b"\r\n").await?;
                stream.read_exact(&mut buf).await?;
                let formatted_resp_time = Utc::now().format("%F %T%.6f");
                let res = String::from_utf8_lossy(&buf);
                let status = res[9..].parse::<u16>()?;
                match status {
                    200 => {
                        writeln!(
                            stdout(),
                            "[{}] {} @ {}",
                            Green.paint("success"),
                            Green.paint("200"),
                            Cyan.paint(format!("{}", formatted_resp_time))
                        )?;
                        Ok(true)
                    }
                    status => {
                        writeln!(
                            stdout(),
                            "[{}] {} @ {}",
                            Red.paint("fail"),
                            Red.paint(format!("{}", status)),
                            Cyan.paint(format!("{}", formatted_resp_time))
                        )?;
                        Ok(false)
                    }
                }
            });
            spread += spread_offset as i64;
            handle_vec.push(handle);
        }
        for handle in handle_vec {
            let status = handle.await??;
            status_vec.push(status);
        }
        Ok(status_vec.contains(&true))
    }
}
