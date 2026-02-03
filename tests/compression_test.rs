//! Compression tests for response decompression
//!
//! Tests for Brotli, Gzip, Deflate, and Zstd decompression.

use spectreq::client::compression::decompress;
use spectreq::CompressionType;

#[test]
fn test_compression_type_from_encoding() {
    assert_eq!(
        CompressionType::from_encoding("gzip"),
        Some(CompressionType::Gzip)
    );
    assert_eq!(
        CompressionType::from_encoding("br"),
        Some(CompressionType::Brotli)
    );
    assert_eq!(
        CompressionType::from_encoding("deflate"),
        Some(CompressionType::Deflate)
    );
    assert_eq!(
        CompressionType::from_encoding("zstd"),
        Some(CompressionType::Zstd)
    );
    assert_eq!(
        CompressionType::from_encoding("identity"),
        Some(CompressionType::None)
    );
    assert_eq!(
        CompressionType::from_encoding(""),
        Some(CompressionType::None)
    );
    assert_eq!(CompressionType::from_encoding("unknown"), None);
}

#[test]
fn test_compression_type_to_encoding_str() {
    assert_eq!(CompressionType::Gzip.to_encoding_str(), "gzip");
    assert_eq!(CompressionType::Brotli.to_encoding_str(), "br");
    assert_eq!(CompressionType::Deflate.to_encoding_str(), "deflate");
    assert_eq!(CompressionType::Zstd.to_encoding_str(), "zstd");
    assert_eq!(CompressionType::None.to_encoding_str(), "identity");
}

#[test]
fn test_identity_passthrough() {
    let data = b"uncompressed data";
    let result = decompress(data, CompressionType::None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data, data);
}

#[test]
fn test_empty_data_decompression() {
    let empty: Vec<u8> = vec![];

    // Identity should handle empty data
    let result = decompress(&empty, CompressionType::None);
    assert!(result.is_ok());
    assert!(result.unwrap().data.is_empty());
}

#[test]
fn test_decompression_wire_size() {
    let data = b"some data that will pass through";
    let result = decompress(data, CompressionType::None);
    assert!(result.is_ok());
    // Wire size should match input size
    assert_eq!(result.unwrap().wire_size, data.len());
}

#[test]
fn test_decompress_invalid_gzip() {
    let invalid = b"not valid gzip data";
    let result = decompress(invalid, CompressionType::Gzip);
    assert!(result.is_err(), "Should fail on invalid gzip data");
}

#[test]
fn test_decompress_invalid_brotli() {
    let invalid = b"not valid brotli data";
    let result = decompress(invalid, CompressionType::Brotli);
    assert!(result.is_err(), "Should fail on invalid brotli data");
}
