//! Error types for Spectre
//!
//! This module defines the error types used throughout the Spectre library.
//! All errors are represented by the [`SpectreError`] enum, and a
//! convenient [`Result<T>`] type alias is provided.
//!
//! # Examples
//!
//! ```rust
//! use spectreq::{Result, SpectreError};
//!
//! fn connect() -> Result<()> {
//!     Err(SpectreError::Connection("timeout".to_string()))
//! }
//! ```
//!
//! # Error Types
//!
//! - `Tls` - TLS handshake or configuration errors
//! - `Http` - HTTP protocol errors
//! - `Connection` - Network connection errors
//! - `InvalidProfile` - Invalid or malformed profile data
//! - `Compression` - Decompression errors
//! - `InvalidUrl` - URL parsing errors
//! - `Timeout` - Request timeout errors
//! - `Io` - Standard I/O errors
//! - `Hyper` - HTTP client errors (from hyper crate)

use thiserror::Error;

/// Error type for Spectre operations
///
/// This enum represents all possible errors that can occur when using
/// the Spectre library for browser impersonation.
#[derive(Error, Debug)]
pub enum SpectreError {
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Invalid profile: {0}")]
    InvalidProfile(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hyper error: {0}")]
    Hyper(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

// Implement From for hyper::Error
impl From<hyper::Error> for SpectreError {
    fn from(err: hyper::Error) -> Self {
        SpectreError::Hyper(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SpectreError>;
