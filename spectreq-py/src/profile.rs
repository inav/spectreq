//! Browser profile wrapper for Python
//!
//! This module provides Python bindings for browser profiles,
//! allowing Python code to use pre-configured browser fingerprinting profiles.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use spectreq::Profile;

/// Python wrapper for browser Profile
///
/// Represents a browser fingerprinting profile with TLS, HTTP/2, TCP,
/// and header configurations that mimic real browsers.
///
/// # Example
///
/// ```python
/// from spectre import Profile
///
/// # Get a pre-configured profile
/// profile = Profile.chrome_143_windows()
/// print(profile.browser)  # "Chrome"
/// print(profile.os)  # "Windows"
/// print(profile.user_agent)  # Full user agent string
///
/// # Available profiles
/// chrome_win = Profile.chrome_143_windows()
/// chrome_mac = Profile.chrome_143_macos()
/// chrome_linux = Profile.chrome_143_linux()
/// firefox_win = Profile.firefox_121_windows()
/// safari_mac = Profile.safari_17_macos()
/// edge_win = Profile.edge_120_windows()
/// ```
#[pyclass(name = "Profile")]
pub struct PyProfile {
    inner: Profile,
}

impl From<Profile> for PyProfile {
    fn from(profile: Profile) -> Self {
        Self { inner: profile }
    }
}

impl From<PyProfile> for Profile {
    fn from(py_profile: PyProfile) -> Self {
        py_profile.inner
    }
}

impl Clone for PyProfile {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[pymethods]
impl PyProfile {
    // Legacy Chrome 120 profiles
    /// Create a Chrome 120 on Windows profile
    #[staticmethod]
    fn chrome_120_windows() -> Self {
        Profile::chrome_120_windows().into()
    }

    /// Create a Chrome 120 on macOS profile
    #[staticmethod]
    fn chrome_120_macos() -> Self {
        Profile::chrome_120_macos().into()
    }

    /// Create a Chrome 120 on Linux profile
    #[staticmethod]
    fn chrome_120_linux() -> Self {
        Profile::chrome_120_linux().into()
    }

    /// Create a Chrome 120 on Android profile
    #[staticmethod]
    fn chrome_120_android() -> Self {
        Profile::chrome_120_android().into()
    }

    // New Chrome 131-143 profiles
    /// Create a Chrome 131 on Windows profile
    #[staticmethod]
    fn chrome_131_windows() -> Self {
        Profile::chrome_131_windows().into()
    }

    /// Create a Chrome 133 on Windows profile
    #[staticmethod]
    fn chrome_133_windows() -> Self {
        Profile::chrome_133_windows().into()
    }

    /// Create a Chrome 141 on Windows profile
    #[staticmethod]
    fn chrome_141_windows() -> Self {
        Profile::chrome_141_windows().into()
    }

    /// Create a Chrome 143 on Windows profile
    #[staticmethod]
    fn chrome_143_windows() -> Self {
        Profile::chrome_143_windows().into()
    }

    /// Create a Chrome 143 on macOS profile
    #[staticmethod]
    fn chrome_143_macos() -> Self {
        Profile::chrome_143_macos().into()
    }

    /// Create a Chrome 143 on Linux profile
    #[staticmethod]
    fn chrome_143_linux() -> Self {
        Profile::chrome_143_linux().into()
    }

    /// Create a Chrome 143 on Android profile
    #[staticmethod]
    fn chrome_143_android() -> Self {
        Profile::chrome_143_android().into()
    }

    /// Firefox 121 on Windows profile
    #[staticmethod]
    fn firefox_121_windows() -> Self {
        Profile::firefox_121_windows().into()
    }

    /// Safari 17 on macOS profile
    #[staticmethod]
    fn safari_17_macos() -> Self {
        Profile::safari_17_macos().into()
    }

    /// Edge 120 on Windows profile
    #[staticmethod]
    fn edge_120_windows() -> Self {
        Profile::edge_120_windows().into()
    }

    /// Get the browser name
    #[getter]
    fn browser(&self) -> String {
        format!("{:?}", self.inner.browser)
    }

    /// Get the operating system
    #[getter]
    fn os(&self) -> String {
        format!("{:?}", self.inner.os)
    }

    /// Get the version string
    #[getter]
    fn version(&self) -> String {
        self.inner.version.clone()
    }

    /// Get the user agent string
    #[getter]
    fn user_agent(&self) -> String {
        self.inner.user_agent.clone()
    }

    /// Get the accept encoding header value
    #[getter]
    fn accept_encoding(&self) -> String {
        self.inner.accept_encoding.clone()
    }

    // ========================================================================
    // Random profile selection (anti-detection)
    // ========================================================================

    /// Get a random browser profile
    ///
    /// Useful for anti-detection by rotating browser fingerprints.
    ///
    /// # Example
    ///
    /// ```python
    /// profile = Profile.random()
    /// print(f"Using {profile.browser} on {profile.os}")
    /// ```
    #[staticmethod]
    fn random() -> Self {
        Profile::random().into()
    }

    /// Get a random Chrome profile
    ///
    /// Returns one of the available Chrome profiles randomly.
    #[staticmethod]
    fn random_chrome() -> Self {
        Profile::random_chrome().into()
    }

    /// Randomize session-specific values for anti-detection
    ///
    /// This randomizes session seed and makes minor HTTP/2 setting variations.
    ///
    /// # Example
    ///
    /// ```python
    /// profile = Profile.chrome_143_windows().randomize()
    /// ```
    fn randomize(&self) -> Self {
        self.inner.clone().randomize().into()
    }

    // ========================================================================
    // Serialization (JSON/YAML)
    // ========================================================================

    /// Load a profile from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file
    ///
    /// # Example
    ///
    /// ```python
    /// profile = Profile.from_json_file("profiles/chrome_143.json")
    /// ```
    #[staticmethod]
    fn from_json_file(path: &str) -> PyResult<Self> {
        Profile::from_json_file(path)
            .map(|p| p.into())
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    /// Load a profile from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML file
    ///
    /// # Example
    ///
    /// ```python
    /// profile = Profile.from_yaml_file("profiles/chrome_143.yaml")
    /// ```
    #[staticmethod]
    fn from_yaml_file(path: &str) -> PyResult<Self> {
        Profile::from_yaml_file(path)
            .map(|p| p.into())
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))
    }

    /// Load a profile from a JSON string
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        Profile::from_json(json)
            .map(|p| p.into())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Load a profile from a YAML string
    #[staticmethod]
    fn from_yaml(yaml: &str) -> PyResult<Self> {
        Profile::from_yaml(yaml)
            .map(|p| p.into())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Export profile to JSON string
    fn to_json(&self) -> PyResult<String> {
        self.inner
            .to_json()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Export profile to YAML string
    fn to_yaml(&self) -> PyResult<String> {
        self.inner
            .to_yaml()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Get profile as a dict for debugging
    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("browser", format!("{:?}", self.inner.browser))?;
        dict.set_item("os", format!("{:?}", self.inner.os))?;
        dict.set_item("version", &self.inner.version)?;
        dict.set_item("user_agent", &self.inner.user_agent)?;
        dict.set_item("accept_encoding", &self.inner.accept_encoding)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "Profile(browser={:?}, os={:?}, version={})",
            self.inner.browser, self.inner.os, self.inner.version
        )
    }

    fn __str__(&self) -> String {
        format!(
            "{} on {} {}",
            self.inner.browser, self.inner.os, self.inner.version
        )
    }
}

/// Helper to expose the inner Profile to other modules
impl PyProfile {
    pub fn inner(&self) -> &Profile {
        &self.inner
    }
}
