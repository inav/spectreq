//! Spectre Core - Browser fingerprinting and HTTP profile library
//!
//! This library provides the core functionality for browser impersonation,
//! including TLS fingerprinting, HTTP/2 settings, TCP configuration, and
//! JA4/JA4H fingerprint calculation.

pub mod ech;
pub mod error;
pub mod headers;
pub mod ja4;
pub mod profile;
pub mod tcp;
pub mod tls;

pub use ech::{
    domain_supports_ech, fetch_ech_configs, parse_ech_config, EchConfig, EchFetchResult,
};
pub use error::{Result, SpectreError};
pub use headers::{
    generate_client_hints, generate_sec_fetch_headers, hashmap_to_ordered, merge_ordered_headers,
    ClientHints, FetchDest, FetchMode, FetchSite, OrderedHeaders, RequestContext,
};
pub use ja4::{
    calculate_ja4, calculate_ja4_from_handshake, Ja4Fingerprint, Ja4RawComponents,
    Ja4hRawComponents,
};
pub use profile::{BrowserName, Headers, Http2Settings, Profile, TcpConfig, TlsConfig, OS};
pub use tcp::{apply_tcp_options, create_tcp_socket};
pub use tls::{build_tls_config, get_ja4_components, supports_post_quantum, Ja4Components};
