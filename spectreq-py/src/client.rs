//! HTTP client wrapper for Python
//!
//! This module provides Python bindings for the Spectre HTTP client,
//! enabling async HTTP requests with browser impersonation from Python.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_async_runtimes::tokio::future_into_py;
use spectreq::Client;
use spectreq::Profile;
use std::collections::HashMap;
use std::sync::Arc;

use crate::cookies::PyCookieJar;
use spectreq::client::RequestTiming;

/// Request timing metrics
#[pyclass(name = "RequestTiming")]
#[derive(Clone)]
pub struct PyRequestTiming {
    #[pyo3(get)]
    pub dns_lookup: f64,
    #[pyo3(get)]
    pub tcp_connect: f64,
    #[pyo3(get)]
    pub tls_handshake: f64,
    #[pyo3(get)]
    pub ttfb: f64,
    #[pyo3(get)]
    pub total: f64,
}

impl From<RequestTiming> for PyRequestTiming {
    fn from(t: RequestTiming) -> Self {
        Self {
            dns_lookup: t.dns_lookup.as_secs_f64(),
            tcp_connect: t.tcp_connect.as_secs_f64(),
            tls_handshake: t.tls_handshake.as_secs_f64(),
            ttfb: t.ttfb.as_secs_f64(),
            total: t.total.as_secs_f64(),
        }
    }
}

/// HTTP response wrapper for Python
///
/// Represents an HTTP response with status, headers, body, and metadata.
///
/// # Example
///
/// ```python
/// response = await client.get("https://httpbin.org/get")
///
/// # Status code
/// print(response.status_code)  # 200
///
/// # Response body
/// text = response.text()  # As string
/// content = response.content  # As bytes
///
/// # JSON response
/// data = response.json()  # Returns dict/list
///
/// # Headers
/// headers = response.headers_dict()  # All headers as dict
/// content_type = response.get_header("content-type")  # Specific header
///
/// # Metadata
/// print(response.wire_size)  # Compressed size
/// print(response.from_cache)  # True if from cache
/// print(response.ok())  # True if 2xx status
/// ```
#[pyclass(name = "Response")]
pub struct PyResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub wire_size: usize,
    pub from_cache: bool,
    pub timing: PyRequestTiming,
}

#[pymethods]
impl PyResponse {
    /// Get the HTTP status code
    #[getter]
    fn status_code(&self) -> u16 {
        self.status
    }

    /// Get the response body as text
    fn text(&self) -> PyResult<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| pyo3::exceptions::PyUnicodeError::new_err(e.to_string()))
    }

    /// Get the response body as bytes
    fn content(&self) -> &[u8] {
        &self.body
    }

    /// Get the wire size (compressed size from network)
    #[getter]
    fn wire_size(&self) -> usize {
        self.wire_size
    }

    /// Check if response was from cache
    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_cache(&self) -> bool {
        self.from_cache
    }

    /// Get request timing metrics
    #[getter]
    fn timing(&self) -> PyRequestTiming {
        self.timing.clone()
    }

    /// Get all headers as a dict
    fn headers_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        for (k, v) in &self.headers {
            dict.set_item(k, v)?;
        }
        Ok(dict.into())
    }

    /// Get a specific header value
    fn get_header(&self, name: &str) -> Option<String> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }

    /// Check if request was successful
    fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Parse response body as JSON
    fn json(&self, py: Python) -> PyResult<Py<PyAny>> {
        let text = self.text()?;
        let json_module = py.import("json")?;
        json_module
            .call_method1("loads", (text,))
            .map(|v| v.unbind())
            .map_err(|e: PyErr| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!(
            "Response(status={}, wire_size={}, from_cache={})",
            self.status, self.wire_size, self.from_cache
        )
    }
}

/// HTTP client wrapper for Python
///
/// Async HTTP client with browser fingerprinting capabilities.
///
/// # Example
///
/// ```python
/// import asyncio
/// from spectre import Profile, Client
///
/// async def main():
///     # Create a client
///     profile = Profile.chrome_143_windows()
///     client = Client(profile)
///
///     # Make requests
///     response = await client.get("https://httpbin.org/get")
///     print(response.text())
///
///     # POST request
///     data = b'{"key": "value"}'
///     response = await client.post("https://httpbin.org/post", data)
///     print(response.json())
///
///     # With proxy
///     client = Client(profile, proxy="socks5://127.0.0.1:1080")
///
///     # With custom headers
///     client = Client(profile, headers={"X-Custom": "value"})
///
/// asyncio.run(main())
/// ```
#[pyclass(name = "Client")]
pub struct PyClient {
    inner: Arc<Client>,
}

/// Helper function to convert Python dict to Rust HashMap
fn py_dict_to_headers(
    _py: Python,
    headers_obj: &Bound<PyAny>,
) -> PyResult<HashMap<String, String>> {
    if let Ok(dict) = headers_obj.cast::<PyDict>() {
        let mut map = HashMap::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let value_str = value.extract::<String>()?;
            map.insert(key_str, value_str);
        }
        Ok(map)
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "headers must be a dict",
        ))
    }
}

#[pymethods]
impl PyClient {
    /// Create a new HTTP client with a browser profile
    ///
    /// # Arguments
    ///
    /// * `profile` - A `Profile` instance for browser impersonation
    /// * `proxy` - Optional proxy URL (e.g., "socks5://127.0.0.1:1080")
    /// * `headers` - Optional dict of custom headers to include with all requests
    ///
    /// # Example
    ///
    /// ```python
    /// from spectre import Profile, Client
    ///
    /// profile = Profile.chrome_143_windows()
    ///
    /// # Basic client
    /// client = Client(profile)
    ///
    /// # With proxy
    /// client = Client(profile, proxy="socks5://127.0.0.1:1080")
    ///
    /// # With custom headers
    /// client = Client(profile, headers={"X-API-Key": "secret"})
    ///
    /// # With both
    /// client = Client(
    ///     profile,
    ///     proxy="socks5://127.0.0.1:1080",
    ///     headers={"X-API-Key": "secret"}
    /// )
    /// ```
    #[new]
    #[pyo3(signature = (profile, proxy=None, headers=None))]
    fn new(
        py: Python,
        profile: &crate::profile::PyProfile,
        proxy: Option<String>,
        headers: Option<Bound<PyAny>>,
    ) -> PyResult<Self> {
        let profile_inner: Profile = profile.clone().into();
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Convert Python headers dict to Rust HashMap
        let custom_headers = if let Some(headers_obj) = headers {
            if headers_obj.is_none() {
                HashMap::new()
            } else {
                py_dict_to_headers(py, &headers_obj)?
            }
        } else {
            HashMap::new()
        };

        let client = rt
            .block_on(Client::with_options(profile_inner, proxy, custom_headers))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(client),
        })
    }

    /// Perform an async GET request
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    ///
    /// # Returns
    ///
    /// A coroutine that resolves to a `Response` object
    ///
    /// # Example
    ///
    /// ```python
    /// response = await client.get("https://httpbin.org/get")
    /// print(response.status_code)
    /// print(response.text())
    /// ```
    fn get<'p>(&self, py: Python<'p>, url: String) -> PyResult<Bound<'p, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let resp = client
                .get(&url)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(PyResponse {
                status: resp.status,
                headers: resp.headers,
                body: resp.body,
                wire_size: resp.wire_size,
                from_cache: resp.from_cache,
                timing: resp.timing.into(),
            })
        })
    }

    /// Perform an async POST request
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    /// * `body` - Optional request body as bytes
    ///
    /// # Returns
    ///
    /// A coroutine that resolves to a `Response` object
    ///
    /// # Example
    ///
    /// ```python
    /// # POST JSON data
    /// data = b'{"key": "value"}'
    /// response = await client.post("https://httpbin.org/post", data)
    /// print(response.json())
    /// ```
    #[pyo3(signature = (url, body=None))]
    fn post<'p>(
        &self,
        py: Python<'p>,
        url: String,
        body: Option<Vec<u8>>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let client = self.inner.clone();
        let body_bytes = body.unwrap_or_default();

        future_into_py(py, async move {
            let resp = client
                .post(&url, body_bytes)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(PyResponse {
                status: resp.status,
                headers: resp.headers,
                body: resp.body,
                wire_size: resp.wire_size,
                from_cache: resp.from_cache,
                timing: resp.timing.into(),
            })
        })
    }

    /// Perform an async PUT request
    #[pyo3(signature = (url, body=None))]
    fn put<'p>(
        &self,
        py: Python<'p>,
        url: String,
        body: Option<Vec<u8>>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let client = self.inner.clone();
        let body_bytes = body.unwrap_or_default();

        future_into_py(py, async move {
            let resp = client
                .put(&url, body_bytes)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(PyResponse {
                status: resp.status,
                headers: resp.headers,
                body: resp.body,
                wire_size: resp.wire_size,
                from_cache: resp.from_cache,
                timing: resp.timing.into(),
            })
        })
    }

    /// Perform an async DELETE request
    fn delete<'p>(&self, py: Python<'p>, url: String) -> PyResult<Bound<'p, PyAny>> {
        let client = self.inner.clone();

        future_into_py(py, async move {
            let resp = client
                .delete(&url)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(PyResponse {
                status: resp.status,
                headers: resp.headers,
                body: resp.body,
                wire_size: resp.wire_size,
                from_cache: resp.from_cache,
                timing: resp.timing.into(),
            })
        })
    }

    /// Perform an async PATCH request
    #[pyo3(signature = (url, body=None))]
    fn patch<'p>(
        &self,
        py: Python<'p>,
        url: String,
        body: Option<Vec<u8>>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let client = self.inner.clone();
        let body_bytes = body.unwrap_or_default();

        future_into_py(py, async move {
            let resp = client
                .patch(&url, body_bytes)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(PyResponse {
                status: resp.status,
                headers: resp.headers,
                body: resp.body,
                wire_size: resp.wire_size,
                from_cache: resp.from_cache,
                timing: resp.timing.into(),
            })
        })
    }

    /// Perform an async HEAD request
    fn head<'p>(&self, py: Python<'p>, url: String) -> PyResult<Bound<'p, PyAny>> {
        let client = self.inner.clone();

        future_into_py(py, async move {
            let resp = client
                .head(&url)
                .await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(PyResponse {
                status: resp.status,
                headers: resp.headers,
                body: resp.body,
                wire_size: resp.wire_size,
                from_cache: resp.from_cache,
                timing: resp.timing.into(),
            })
        })
    }

    /// Get the proxy configuration
    #[getter]
    fn proxy(&self) -> Option<String> {
        self.inner.proxy().map(|s| s.to_string())
    }

    /// Get the custom headers as a dict
    #[getter]
    fn headers(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        for (key, value) in self.inner.headers() {
            dict.set_item(key, value)?;
        }
        Ok(dict.into())
    }

    /// Get the cookie jar
    fn cookie_jar(&self) -> PyCookieJar {
        PyCookieJar {
            inner: self.inner.cookie_jar().clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!("Client(profile={:?})", self.inner.profile())
    }
}
