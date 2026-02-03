use http::uri::Scheme;
use hyper::rt::{Read as HyperRead, Write as HyperWrite};
use hyper::Uri;
use hyper_util::client::legacy::connect::{Connected, Connection};
use crate::core::{build_tls_config, create_tcp_socket, Profile, SpectreError};
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tower::Service;

// Inline URL parsing for proxy support
fn parse_proxy_url(url: &str) -> Option<(String, u16)> {
    let url_obj = url::Url::parse(url).ok()?;
    let host = url_obj.host_str()?.to_string();
    let port = url_obj.port_or_known_default()?;
    Some((host, port))
}

/// Custom stream type that can be either a plain TCP stream or a TLS stream
pub enum MaybeTlsStream {
    Plain(TcpStream),
    Tls(tokio_rustls::client::TlsStream<TcpStream>),
}

impl AsyncRead for MaybeTlsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Plain(s) => Pin::new(s).poll_read(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for MaybeTlsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            MaybeTlsStream::Plain(s) => Pin::new(s).poll_write(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Plain(s) => Pin::new(s).poll_flush(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Plain(s) => Pin::new(s).poll_shutdown(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connection wrapper that implements both tokio and hyper IO traits
pub struct ImpersonateConnection {
    inner: MaybeTlsStream,
}

impl ImpersonateConnection {
    pub fn new(stream: MaybeTlsStream) -> Self {
        Self { inner: stream }
    }

    pub fn plain(stream: TcpStream) -> Self {
        Self::new(MaybeTlsStream::Plain(stream))
    }

    pub fn tls(stream: tokio_rustls::client::TlsStream<TcpStream>) -> Self {
        Self::new(MaybeTlsStream::Tls(stream))
    }
}

// Tokio IO traits
impl AsyncRead for ImpersonateConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for ImpersonateConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// Hyper IO traits
impl HyperRead for ImpersonateConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        let mut tmp = vec![0u8; buf.remaining().min(8192)];
        let mut read_buf = ReadBuf::new(&mut tmp);

        match Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let n = read_buf.filled().len();
                if n > 0 {
                    buf.put_slice(&tmp[..n]);
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl HyperWrite for ImpersonateConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl Connection for ImpersonateConnection {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

/// Custom connector that applies TCP options and TLS configuration based on profile
#[derive(Clone)]
pub struct ImpersonateConnector {
    profile: Profile,
    #[allow(dead_code)]
    tls_config: Arc<rustls::ClientConfig>, // Reserved for future use
    tls_connector: Arc<TlsConnector>,
    proxy: Option<String>,
}

impl ImpersonateConnector {
    /// Create a new connector with the given profile
    pub fn new(profile: Profile) -> std::result::Result<Self, SpectreError> {
        Self::with_proxy(profile, None)
    }

    /// Create a new connector with the given profile and optional proxy
    pub fn with_proxy(
        profile: Profile,
        proxy: Option<String>,
    ) -> std::result::Result<Self, SpectreError> {
        let tls_config = build_tls_config(&profile)?;
        let tls_connector = TlsConnector::from(Arc::new(tls_config.clone()));
        Ok(Self {
            profile,
            tls_config: Arc::new(tls_config),
            tls_connector: Arc::new(tls_connector),
            proxy,
        })
    }

    /// Get the profile associated with this connector
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Get the proxy URL if set
    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }

    /// Parse proxy URL to get (host, port)
    fn parse_proxy_url(&self) -> Option<(String, u16)> {
        self.proxy.as_ref().and_then(|p| parse_proxy_url(p))
    }

    /// Connect to a destination with TCP options applied
    async fn connect_tcp(&self, host: &str, port: u16) -> io::Result<TcpStream> {
        use socket2::{Domain, SockAddr};
        use std::net::ToSocketAddrs;

        // Resolve the address
        let addr = format!("{}:{}", host, port);
        let addrs = addr.to_socket_addrs()?.collect::<Vec<_>>();
        if addrs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No addresses found",
            ));
        }

        // Use the first address
        let sock_addr = addrs[0];

        // Determine the domain
        let domain = if sock_addr.is_ipv6() {
            Domain::IPV6
        } else {
            Domain::IPV4
        };

        // Create and configure the socket with profile options using socket2
        let socket = create_tcp_socket(&self.profile, domain)?;

        // Set non-blocking mode for async connection
        socket.set_nonblocking(true)?;

        // Initiate connection using socket2
        // On non-blocking sockets, connect returns WouldBlock if connection is in progress
        match socket.connect(&SockAddr::from(sock_addr)) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.raw_os_error() == Some(115) => {
                // Connection in progress (EINPROGRESS = 115 on Linux)
                // This is expected for non-blocking sockets
            }
            Err(e) => return Err(e),
        }

        // Convert to tokio TcpStream
        // Note: The connection is in progress, tokio will handle completing it
        let std_stream: std::net::TcpStream = socket.into();
        TcpStream::from_std(std_stream)
    }

    /// Connect using TLS
    async fn connect_tls(
        &self,
        host: &str,
        port: u16,
    ) -> io::Result<tokio_rustls::client::TlsStream<TcpStream>> {
        // First establish TCP connection (use connect_to if set for domain fronting)
        let connect_host = self.profile.connect_to.as_deref().unwrap_or(host);
        let tcp_stream = self.connect_tcp(connect_host, port).await?;

        // Extract SNI from hostname (use sni_override if set for domain fronting)
        let sni_host = self.profile.sni_override.as_deref().unwrap_or(host);
        let dns_name = rustls::pki_types::ServerName::try_from(sni_host.to_string())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid server name"))?;

        // Perform TLS handshake
        let tls_stream = self.tls_connector.connect(dns_name, tcp_stream).await?;

        Ok(tls_stream)
    }
}

impl Service<Uri> for ImpersonateConnector {
    type Response = ImpersonateConnection;
    type Error = SpectreError;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Uri) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            let host = req
                .host()
                .ok_or_else(|| SpectreError::Http("Missing host in URI".to_string()))?;

            let port = req.port_u16().unwrap_or_else(|| {
                if req.scheme() == Some(&Scheme::HTTPS) {
                    443
                } else {
                    80
                }
            });

            let is_https = req.scheme() == Some(&Scheme::HTTPS);

            // For domain fronting: get the actual host to connect to
            let connect_host = this.profile.connect_to.as_deref().unwrap_or(host);
            // For domain fronting: get the SNI hostname
            let sni_host = this.profile.sni_override.as_deref().unwrap_or(host);

            // If proxy is set, use CONNECT for HTTPS or direct connection for HTTP
            if let Some((proxy_host, proxy_port)) = this.parse_proxy_url() {
                if is_https {
                    // HTTPS through proxy: use HTTP CONNECT
                    // First connect to proxy
                    let proxy_stream =
                        this.connect_tcp(&proxy_host, proxy_port)
                            .await
                            .map_err(|e| {
                                SpectreError::Http(format!("Proxy connection failed: {}", e))
                            })?;

                    // Send CONNECT request (connect to the connect_host for domain fronting)
                    let connect_request = format!(
                        "CONNECT {connect_host}:{port} HTTP/1.1\r\n\
                         Host: {connect_host}:{port}\r\n\
                         User-Agent: {}\r\n\
                         Proxy-Connection: keep-alive\r\n\r\n",
                        this.profile.user_agent
                    );

                    // Use a simpler approach: write, read response, then use the stream
                    let mut proxy_stream = proxy_stream;

                    // Write CONNECT request
                    tokio::io::AsyncWriteExt::write_all(
                        &mut proxy_stream,
                        connect_request.as_bytes(),
                    )
                    .await
                    .map_err(|e| SpectreError::Http(format!("Failed to send CONNECT: {}", e)))?;

                    // Read response until we get "\r\n\r\n"
                    let mut buffer = vec![0u8; 4096];
                    let mut total_read = 0;
                    let response_end = loop {
                        let n = tokio::io::AsyncReadExt::read(
                            &mut proxy_stream,
                            &mut buffer[total_read..],
                        )
                        .await
                        .map_err(|e| {
                            SpectreError::Http(format!("Failed to read CONNECT response: {}", e))
                        })?;
                        if n == 0 {
                            return Err(SpectreError::Http("Proxy closed connection".to_string()));
                        }
                        total_read += n;
                        if buffer[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
                            break total_read;
                        }
                        if total_read >= buffer.len() {
                            return Err(SpectreError::Http(
                                "CONNECT response too large".to_string(),
                            ));
                        }
                    };

                    // Check for "200" response
                    let response = String::from_utf8_lossy(&buffer[..response_end]);
                    if !response.contains("200") {
                        return Err(SpectreError::Http(format!("CONNECT failed: {}", response)));
                    }

                    // Now establish TLS through the proxy tunnel (use sni_host for SNI)
                    let dns_name = rustls::pki_types::ServerName::try_from(sni_host.to_string())
                        .map_err(|_| {
                            io::Error::new(io::ErrorKind::InvalidInput, "Invalid server name")
                        })?;

                    let tls_stream = this.tls_connector.connect(dns_name, proxy_stream).await?;

                    return Ok(ImpersonateConnection::tls(tls_stream));
                } else {
                    // HTTP through proxy: connect to proxy, hyper will handle the rest
                    let tcp_stream =
                        this.connect_tcp(&proxy_host, proxy_port)
                            .await
                            .map_err(|e| {
                                SpectreError::Http(format!("Proxy connection failed: {}", e))
                            })?;
                    return Ok(ImpersonateConnection::plain(tcp_stream));
                }
            }

            // Direct connection (no proxy)
            if is_https {
                let tls_stream = this
                    .connect_tls(host, port)
                    .await
                    .map_err(|e| SpectreError::Http(format!("TLS connection failed: {}", e)))?;
                Ok(ImpersonateConnection::tls(tls_stream))
            } else {
                let tcp_stream = this
                    .connect_tcp(host, port)
                    .await
                    .map_err(|e| SpectreError::Http(format!("TCP connection failed: {}", e)))?;
                Ok(ImpersonateConnection::plain(tcp_stream))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_create() {
        let profile = Profile::chrome_120_windows();
        let connector = ImpersonateConnector::new(profile);
        assert!(connector.is_ok());
    }

    #[test]
    fn test_connector_profile() {
        let profile = Profile::chrome_120_windows();
        let connector = ImpersonateConnector::new(profile).unwrap();
        assert_eq!(
            connector.profile().browser,
            crate::core::BrowserName::Chrome
        );
    }

    #[test]
    fn test_connector_with_proxy() {
        let profile = Profile::chrome_120_windows();
        let connector = ImpersonateConnector::with_proxy(
            profile,
            Some("http://proxy.example.com:8080".to_string()),
        );
        assert!(connector.is_ok());
        let connector = connector.unwrap();
        assert_eq!(connector.proxy(), Some("http://proxy.example.com:8080"));
    }

    #[test]
    fn test_connector_proxy_none() {
        let profile = Profile::chrome_120_windows();
        let connector = ImpersonateConnector::with_proxy(profile, None).unwrap();
        assert_eq!(connector.proxy(), None);
    }

    #[test]
    fn test_connector_clone() {
        let profile = Profile::chrome_120_windows();
        let connector = ImpersonateConnector::new(profile).unwrap();
        let cloned = connector.clone();
        assert_eq!(connector.profile().browser, cloned.profile().browser);
    }
}
