use async_std::net::TcpStream;
use async_std::{io, task};
use async_tls::client::TlsStream;
use async_tls::TlsConnector;
use futures::future;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

// Use this if HTTP (port is 80)
pub struct ConnectionManager {
    address: SocketAddr,
    streams: Vec<TcpStream>,
}

impl ConnectionManager {
    pub fn new(host: String, port: u32) -> Self {
        let addr_string = format!("{}:{}", host, port);
        ConnectionManager {
            address: addr_string.to_socket_addrs().unwrap().next().unwrap(),
            streams: Vec::new(),
        }
    }

    // Connections => number of attempts to send request
    pub fn connect(&mut self, connections: u32) {
        async fn create_stream(address: SocketAddr) -> Option<TcpStream> {
            let stream = TcpStream::connect(address).await;
            stream.ok()
        }

        task::block_on(async {
            self.streams.extend(
                future::join_all((0..connections).map(|_| create_stream(self.address)))
                    .await
                    .into_iter()
                    .filter_map(|s| s),
            );
        });
    }

    // Payload are to be encoded in UTF-8 bytes as a byte array
    pub fn send(&mut self, payloads: Vec<&[u8]>) {
        let mut payloads = payloads.iter().cycle();
        task::block_on(future::join_all(
            self.streams
                .iter_mut()
                .map(|s| s.write_all(payloads.next().unwrap())),
        ));
    }

    pub fn recieve(&mut self, timeout: u64) -> Vec<Vec<u8>> {
        let mut bufs: Vec<Vec<u8>> = self.streams.iter().map(|_| Vec::new()).collect();
        task::block_on(future::join_all(
            self.streams
                .iter_mut()
                .zip(bufs.iter_mut())
                .map(|(s, b)| io::timeout(Duration::from_secs(timeout), s.read_to_end(b))),
        ));
        bufs
    }
}

// Use this if HTTPS (port is 443)
pub struct TLSConnectionManager {
    address: SocketAddr,
    domain: String,
    streams: Vec<TlsStream<TcpStream>>,
}

impl TLSConnectionManager {
    pub fn new(host: String, port: u32, domain: String) -> Self {
        let addr_string = format!("{}:{}", host, port);
        TLSConnectionManager {
            address: addr_string.to_socket_addrs().unwrap().next().unwrap(),
            streams: Vec::new(),
            domain,
        }
    }

    // Connections => number of attempts to send request
    pub fn connect(&mut self, connections: u32) {
        let connector = TlsConnector::default();
        async fn create_stream(
            connector: &TlsConnector,
            address: SocketAddr,
            domain: &str,
        ) -> Option<TlsStream<TcpStream>> {
            let stream = TcpStream::connect(address).await.unwrap();
            connector.connect(domain, stream).await.ok()
        }

        self.streams.reserve(connections as usize);
        task::block_on(async {
            self.streams.extend(
                future::join_all(
                    (0..connections).map(|_| create_stream(&connector, self.address, &self.domain)),
                )
                .await
                .into_iter()
                .filter_map(|s| s),
            );
        });
    }

    // Payload are to be encoded in UTF-8 bytes as a byte array
    pub fn send(&mut self, payloads: Vec<&[u8]>) {
        let mut payloads = payloads.iter().cycle();
        task::block_on(future::join_all(
            self.streams
                .iter_mut()
                .map(|s| s.write_all(payloads.next().unwrap())),
        ));
    }

    pub fn recieve(&mut self, timeout: u64) -> Vec<Vec<u8>> {
        let mut bufs: Vec<Vec<u8>> = self.streams.iter().map(|_| Vec::new()).collect();
        task::block_on(future::join_all(
            self.streams
                .iter_mut()
                .zip(bufs.iter_mut())
                .map(|(s, b)| io::timeout(Duration::from_secs(timeout), s.read_to_end(b))),
        ));
        bufs
    }
}
