use pyo3::prelude::*;
use spectreq::client::cookies::CookieJar;
use url::Url;

/// CookieJar for managing cookies
///
/// Handles storage and automatic inclusion of cookies in requests.
///
/// # Example
///
/// ```python
/// jar = client.cookie_jar()
///
/// # Set cookies
/// jar.set_cookies("https://example.com", ["session=123"])
///
/// # Get cookies
/// cookies = jar.get_cookie_value("https://example.com")
/// print(cookies)  # "session=123"
///
/// # Clear
/// jar.clear()
/// ```
#[pyclass(name = "CookieJar")]
pub struct PyCookieJar {
    pub(crate) inner: CookieJar,
}

#[pymethods]
impl PyCookieJar {
    /// Create a new CookieJar
    #[new]
    fn new() -> Self {
        Self {
            inner: CookieJar::new(),
        }
    }

    /// Set cookies for a specific URL
    ///
    /// The cookies should be in "Set-Cookie" header format.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL associated with the cookies
    /// * `cookies` - List of cookie strings
    fn set_cookies(&self, url: String, cookies: Vec<String>) -> PyResult<()> {
        let url = Url::parse(&url)
            .map_err(|e: url::ParseError| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        // Convert Vec<String> to Vec<&str> for the underlying API
        let refs: Vec<&str> = cookies.iter().map(|s| s.as_str()).collect();
        self.inner.set_cookies(&refs, &url);
        
        Ok(())
    }

    /// Get cookies for a URL as a Cookie message header value
    ///
    /// Returns the "Cookie" header value string, or None if no cookies match.
    fn get_cookie_value(&self, url: String) -> PyResult<Option<String>> {
        let url = Url::parse(&url)
            .map_err(|e: url::ParseError| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
            
        Ok(self.inner.get_cookie_value(&url))
    }

    /// Clear all cookies
    fn clear(&self) {
        self.inner.clear();
    }

    /// Get number of cookies
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove cookies for a specific domain
    fn remove_for_domain(&self, domain: String) {
        self.inner.remove_for_domain(&domain);
    }
    
    fn __len__(&self) -> usize {
        self.inner.len()
    }
    
    fn __repr__(&self) -> String {
        format!("CookieJar(count={})", self.inner.len())
    }
}
