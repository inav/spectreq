//! SOCKS5 proxy support
//!
//! This module provides SOCKS5 proxy support for the Spectre client,
//! allowing connections through SOCKS5 proxies with optional authentication.

use rustls::ClientConfig;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

/// SOCKS5 proxy configuration
#[derive(Debug, Clone)]
pub struct Socks5Config {
    /// Proxy server address
    pub proxy_addr: SocketAddr,
    /// Optional username for authentication
    pub username: Option<String>,
    /// Optional password for authentication
    pub password: Option<String>,
    /// DNS resolution method
    pub dns_resolve: Socks5DnsResolve,
}

/// DNS resolution method for SOCKS5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Socks5DnsResolve {
    /// Resolve DNS locally (connect to IP, send hostname to proxy)
    Local,
    /// Resolve DNS through proxy (send hostname to proxy)
    Proxy,
}

impl Socks5Config {
    /// Create a new SOCKS5 config for a proxy address
    pub fn new(proxy_addr: SocketAddr) -> Self {
        Self {
            proxy_addr,
            username: None,
            password: None,
            dns_resolve: Socks5DnsResolve::Local,
        }
    }

    /// Set authentication
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    /// Set DNS resolution method
    pub fn with_dns_resolve(mut self, method: Socks5DnsResolve) -> Self {
        self.dns_resolve = method;
        self
    }

    /// Check if authentication is configured
    pub fn has_auth(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }
}

/// SOCKS5 connector
pub struct Socks5Connector {
    config: Socks5Config,
    tls_config: Option<Arc<ClientConfig>>,
    tls_connector: Option<Arc<TlsConnector>>,
}

impl Socks5Connector {
    /// Create a new SOCKS5 connector
    pub fn new(config: Socks5Config) -> Self {
        Self {
            config,
            tls_config: None,
            tls_connector: None,
        }
    }

    /// Create a new SOCKS5 connector with TLS support
    pub fn with_tls(mut self, tls_config: Arc<ClientConfig>) -> Self {
        self.tls_config = Some(tls_config.clone());
        self.tls_connector = Some(Arc::new(TlsConnector::from(tls_config)));
        self
    }

    /// Connect to a target host through the SOCKS5 proxy
    pub async fn connect(&self, host: &str, port: u16) -> io::Result<TcpStream> {
        // Connect to the SOCKS5 proxy
        let mut stream = TcpStream::connect(self.config.proxy_addr).await?;

        // SOCKS5 handshake
        self.handshake(&mut stream).await?;

        // SOCKS5 CONNECT request
        self.connect_request(&mut stream, host, port).await?;

        Ok(stream)
    }

    /// Perform SOCKS5 handshake
    async fn handshake(&self, stream: &mut TcpStream) -> io::Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Build greeting
        let num_methods = if self.config.has_auth() { 2 } else { 1 };
        let greeting = [
            0x05,        // SOCKS version
            num_methods, // Number of methods
            0x00,        // No authentication
            0x02,        // Username/password
        ];

        // Send greeting
        stream.write_all(&greeting[..=num_methods as usize]).await?;

        // Read response
        let mut response = [0u8; 2];
        stream.read_exact(&mut response).await?;

        // Check version
        if response[0] != 0x05 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid SOCKS version in response",
            ));
        }

        // Check selected method
        match response[1] {
            0x00 => {
                // No authentication, nothing more to do
                if self.config.has_auth() {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Server doesn't support authentication",
                    ));
                }
            }
            0x02 => {
                // Username/password authentication
                if !self.config.has_auth() {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Server requires authentication",
                    ));
                }
                self.authenticate(stream).await?;
            }
            0xFF => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "No acceptable authentication methods",
                ));
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown authentication method",
                ));
            }
        }

        Ok(())
    }

    /// Perform username/password authentication
    async fn authenticate(&self, stream: &mut TcpStream) -> io::Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let username = self.config.username.as_ref().unwrap();
        let password = self.config.password.as_ref().unwrap();

        // Build auth request
        let mut auth_request = vec![
            0x01, // Auth version
            username.len() as u8,
        ];
        auth_request.extend_from_slice(username.as_bytes());
        auth_request.push(password.len() as u8);
        auth_request.extend_from_slice(password.as_bytes());

        // Send auth request
        stream.write_all(&auth_request).await?;

        // Read response
        let mut response = [0u8; 2];
        stream.read_exact(&mut response).await?;

        // Check response
        if response[0] != 0x01 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid auth response version",
            ));
        }

        if response[1] != 0x00 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Authentication failed",
            ));
        }

        Ok(())
    }

    /// Send SOCKS5 CONNECT request
    async fn connect_request(
        &self,
        stream: &mut TcpStream,
        host: &str,
        port: u16,
    ) -> io::Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut request = vec![
            0x05, // SOCKS version
            0x01, // CONNECT command
            0x00, // Reserved
        ];

        // Address type and address
        if self.config.dns_resolve == Socks5DnsResolve::Local {
            // Resolve locally and send IP
            let addr = format!("{}:{}", host, port);
            let socket_addr = addr
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No addresses found"))?;

            match socket_addr {
                SocketAddr::V4(addr) => {
                    request.push(0x01); // IPv4
                    request.extend_from_slice(&addr.ip().octets());
                }
                SocketAddr::V6(addr) => {
                    request.push(0x04); // IPv6
                    request.extend_from_slice(&addr.ip().octets());
                }
            }
        } else {
            // Resolve through proxy (send hostname)
            request.push(0x03); // Domain name
            if host.len() > 255 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Hostname too long",
                ));
            }
            request.push(host.len() as u8);
            request.extend_from_slice(host.as_bytes());
        }

        // Port
        request.extend_from_slice(&port.to_be_bytes());

        // Send request
        stream.write_all(&request).await?;

        // Read response
        let mut response = [0u8; 4];
        stream.read_exact(&mut response).await?;

        // Check version
        if response[0] != 0x05 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid SOCKS version in response",
            ));
        }

        // Check reply code
        match response[1] {
            0x00 => {} // Success
            0x01 => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "General SOCKS server failure",
                ));
            }
            0x02 => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Connection not allowed by ruleset",
                ));
            }
            0x03 => {
                return Err(io::Error::new(
                    io::ErrorKind::NetworkUnreachable,
                    "Network unreachable",
                ));
            }
            0x04 => {
                return Err(io::Error::new(
                    io::ErrorKind::HostUnreachable,
                    "Host unreachable",
                ));
            }
            0x05 => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "Connection refused",
                ));
            }
            0x06 => {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "TTL expired"));
            }
            0x07 => {
                return Err(io::Error::other("Command not supported"));
            }
            0x08 => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrNotAvailable,
                    "Address type not supported",
                ));
            }
            _ => {
                return Err(io::Error::other(format!(
                    "Unknown SOCKS error code: {}",
                    response[1]
                )));
            }
        }

        // Read and discard the bound address
        match response[3] {
            0x01 => {
                // IPv4: 4 bytes + 2 bytes port
                let mut addr = [0u8; 6];
                stream.read_exact(&mut addr).await?;
            }
            0x03 => {
                // Domain: 1 byte length + domain + 2 bytes port
                let mut len = [0u8; 1];
                stream.read_exact(&mut len).await?;
                let mut addr = vec![0u8; len[0] as usize + 2];
                stream.read_exact(&mut addr).await?;
            }
            0x04 => {
                // IPv6: 16 bytes + 2 bytes port
                let mut addr = [0u8; 18];
                stream.read_exact(&mut addr).await?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid address type in response",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socks5_config_new() {
        let config = Socks5Config::new("127.0.0.1:1080".parse().unwrap());
        assert!(!config.has_auth());
        assert_eq!(config.dns_resolve, Socks5DnsResolve::Local);
    }

    #[test]
    fn test_socks5_config_with_auth() {
        let config = Socks5Config::new("127.0.0.1:1080".parse().unwrap())
            .with_auth("user".to_string(), "pass".to_string());
        assert!(config.has_auth());
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
    }
}
