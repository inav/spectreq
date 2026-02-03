//! TCP socket configuration for browser impersonation
//!
//! This module provides functions to configure TCP sockets with
//! browser-specific options for fingerprinting purposes.
//!
//! # Examples
//!
//! ```rust,ignore
//! use spectreq::{Profile, create_tcp_socket};
//! use socket2::Domain;
//!
//! let profile = Profile::chrome_143_windows();
//! let socket = create_tcp_socket(&profile, Domain::IPV4)?;
//! // Use socket for connection...
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::core::profile::Profile;
use socket2::{Domain, Protocol, Socket, Type};
use std::time::Duration;

/// Apply TCP options to a socket based on the profile configuration
pub fn apply_tcp_options(socket: &Socket, profile: &Profile) -> std::io::Result<()> {
    let tcp = &profile.tcp;

    // Set TTL if specified
    if let Some(ttl) = tcp.ttl {
        socket.set_ttl(ttl)?;
    }

    // Set window size if specified
    if let Some(window_size) = tcp.window_size {
        socket.set_recv_buffer_size(window_size as usize)?;
        socket.set_send_buffer_size(window_size as usize)?;
    }

    // Enable/disable SACK (Selective Acknowledgment)
    // Note: SACK is typically enabled by default on modern systems
    // and may not be directly configurable via socket2

    // Configure keepalive if enabled
    if tcp.keepalive {
        #[cfg(unix)]
        {
            use socket2::SockRef;
            let sock_ref = SockRef::from(socket);
            let mut ka = socket2::TcpKeepalive::new();
            ka = ka.with_time(Duration::from_secs(
                tcp.keepalive_time_secs.unwrap_or(60) as u64
            ));
            sock_ref.set_tcp_keepalive(&ka)?;
        }
    }

    // Set common options for HTTP client
    socket.set_nodelay(true)?; // Disable Nagle's algorithm

    Ok(())
}

/// Create a new TCP socket configured for the given profile
pub fn create_tcp_socket(profile: &Profile, domain: Domain) -> std::io::Result<Socket> {
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    apply_tcp_options(&socket, profile)?;
    Ok(socket)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_tcp_options() {
        let profile = Profile::chrome_120_windows();
        let socket = create_tcp_socket(&profile, Domain::IPV4);
        assert!(socket.is_ok());
    }
}
