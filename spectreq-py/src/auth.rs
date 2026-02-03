//! Authentication wrappers for Python
//!
//! This module provides Python bindings for authentication helpers.

use pyo3::prelude::*;
use spectreq::{BasicAuth, BearerToken, DigestAuth};
use std::time::Duration;

/// Python wrapper for Bearer Token
///
/// # Example
///
/// ```python
/// from spectreq import BearerToken
///
/// # Simple token
/// token = BearerToken("eyJhbGciOiJIUzI1NiI...")
/// print(token.authorization_header())  # "Bearer eyJ..."
///
/// # Token with expiration (seconds)
/// token = BearerToken.with_expiration("token", 3600)
/// print(token.is_expired())  # False
/// ```
#[pyclass(name = "BearerToken")]
pub struct PyBearerToken {
    inner: BearerToken,
}

#[pymethods]
impl PyBearerToken {
    /// Create a new bearer token
    #[new]
    fn new(access_token: &str) -> Self {
        Self {
            inner: BearerToken::new(access_token),
        }
    }

    /// Create a bearer token with expiration (in seconds)
    #[staticmethod]
    fn with_expiration(access_token: &str, expires_in_secs: u64) -> Self {
        Self {
            inner: BearerToken::with_expiration(access_token, Duration::from_secs(expires_in_secs)),
        }
    }

    /// Create a bearer token with refresh token
    #[staticmethod]
    fn with_refresh_token(access_token: &str, refresh_token: &str, expires_in_secs: u64) -> Self {
        Self {
            inner: BearerToken::with_refresh_token(
                access_token,
                refresh_token,
                Duration::from_secs(expires_in_secs),
            ),
        }
    }

    /// Check if the token is expired
    fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    /// Check if the token will expire within the given seconds
    fn expires_soon(&self, within_secs: u64) -> bool {
        self.inner.expires_soon(Duration::from_secs(within_secs))
    }

    /// Get the authorization header value (e.g., "Bearer token123")
    fn authorization_header(&self) -> String {
        self.inner.authorization_header()
    }

    fn __repr__(&self) -> String {
        format!("BearerToken(expired={})", self.inner.is_expired())
    }
}

/// Python wrapper for Basic Authentication
///
/// # Example
///
/// ```python
/// from spectreq import BasicAuth
///
/// auth = BasicAuth("username", "password")
/// header = auth.authorization_header()
/// print(header)  # "Basic dXNlcm5hbWU6cGFzc3dvcmQ="
/// ```
#[pyclass(name = "BasicAuth")]
pub struct PyBasicAuth {
    inner: BasicAuth,
}

#[pymethods]
impl PyBasicAuth {
    /// Create new basic auth credentials
    #[new]
    fn new(username: &str, password: &str) -> Self {
        Self {
            inner: BasicAuth::new(username, password),
        }
    }

    /// Get the authorization header value
    fn authorization_header(&self) -> String {
        self.inner.authorization_header()
    }

    fn __repr__(&self) -> String {
        "BasicAuth(****)".to_string()
    }
}

/// Python wrapper for Digest Authentication
///
/// # Example
///
/// ```python
/// from spectreq import DigestAuth
///
/// auth = DigestAuth("username", "password")
/// header = auth.authorization_header(
///     method="GET",
///     uri="/protected",
///     realm="example.com",
///     nonce="abc123",
///     qop="auth",
///     opaque=None,
///     nc=1
/// )
/// print(header)  # "Digest username=..."
/// ```
#[pyclass(name = "DigestAuth")]
pub struct PyDigestAuth {
    inner: DigestAuth,
}

#[pymethods]
impl PyDigestAuth {
    /// Create new digest auth credentials
    #[new]
    fn new(username: &str, password: &str) -> Self {
        Self {
            inner: DigestAuth::new(username, password),
        }
    }

    /// Generate the authorization header for a request
    #[pyo3(signature = (method, uri, realm, nonce, qop=None, opaque=None, nc=1))]
    fn authorization_header(
        &self,
        method: &str,
        uri: &str,
        realm: &str,
        nonce: &str,
        qop: Option<&str>,
        opaque: Option<&str>,
        nc: u32,
    ) -> String {
        self.inner
            .authorization_header(method, uri, realm, nonce, qop, opaque, nc)
    }

    fn __repr__(&self) -> String {
        "DigestAuth(****)".to_string()
    }
}
