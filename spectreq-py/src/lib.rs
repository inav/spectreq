//! Spectre Python bindings
//!
//! This crate provides Python bindings for the Spectre HTTP client with
//! browser impersonation capabilities.

use pyo3::prelude::*;

mod auth;
mod client;
mod cookies;
mod profile;

use auth::{PyBearerToken, PyBasicAuth, PyDigestAuth};
use client::{PyClient, PyResponse, PyRequestTiming};
use cookies::PyCookieJar;
use profile::PyProfile;

/// Spectre - Python bindings for browser impersonation HTTP client
///
/// This module provides Python bindings for the Spectre library,
/// which allows making HTTP requests that mimic real browser fingerprints.
///
/// # Example
///
/// ```python
/// import asyncio
/// from spectreq import Profile, Client
///
/// async def main():
///     profile = Profile.chrome_143_windows()
///     client = Client(profile)
///     response = await client.get("https://httpbin.org/get")
///     print(response.text())
///
/// asyncio.run(main())
/// ```
#[pymodule]
fn spectreq(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Profile and Client
    m.add_class::<PyProfile>()?;
    m.add_class::<PyClient>()?;
    m.add_class::<PyResponse>()?;
    m.add_class::<PyRequestTiming>()?;
    m.add_class::<PyCookieJar>()?;
    
    // Authentication
    m.add_class::<PyBearerToken>()?;
    m.add_class::<PyBasicAuth>()?;
    m.add_class::<PyDigestAuth>()?;
    
    Ok(())
}
