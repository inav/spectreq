use crate::core::SpectreError;
use brotli::Decompressor as BrotliDecompressor;
use flate2::read::GzDecoder;
use std::io::Read;

/// Supported compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    Brotli,
    Gzip,
    Deflate,
    Zstd,
    None,
}

impl CompressionType {
    /// Parse compression type from Content-Encoding header value
    pub fn from_encoding(encoding: &str) -> Option<Self> {
        match encoding.to_lowercase().trim() {
            "br" => Some(CompressionType::Brotli),
            "gzip" | "x-gzip" => Some(CompressionType::Gzip),
            "deflate" => Some(CompressionType::Deflate),
            "zstd" => Some(CompressionType::Zstd),
            "identity" | "none" | "" => Some(CompressionType::None),
            _ => None,
        }
    }

    /// Get the encoding string for Accept-Encoding header
    pub fn to_encoding_str(self) -> &'static str {
        match self {
            CompressionType::Brotli => "br",
            CompressionType::Gzip => "gzip",
            CompressionType::Deflate => "deflate",
            CompressionType::Zstd => "zstd",
            CompressionType::None => "identity",
        }
    }
}

/// Decompression result with metadata
#[derive(Debug)]
pub struct Decompressed {
    /// Decompressed data
    pub data: Vec<u8>,
    /// Original wire size (compressed)
    pub wire_size: usize,
    /// Compression type used
    pub compression: CompressionType,
}

/// Decompress data based on compression type
pub fn decompress(data: &[u8], compression: CompressionType) -> Result<Decompressed, SpectreError> {
    let wire_size = data.len();

    let decompressed = match compression {
        CompressionType::Brotli => {
            let mut decoder = BrotliDecompressor::new(data, data.len() * 2);
            let mut output = Vec::with_capacity(data.len());
            decoder.read_to_end(&mut output).map_err(|e| {
                SpectreError::Compression(format!("Brotli decompression failed: {}", e))
            })?;
            output
        }
        CompressionType::Gzip | CompressionType::Deflate => {
            let mut decoder = GzDecoder::new(data);
            let mut output = Vec::with_capacity(data.len());
            decoder.read_to_end(&mut output).map_err(|e| {
                SpectreError::Compression(format!("Gzip/Deflate decompression failed: {}", e))
            })?;
            output
        }
        CompressionType::Zstd => {
            let mut decoder = zstd::stream::Decoder::new(data).map_err(|e| {
                SpectreError::Compression(format!("Zstd decompression failed: {}", e))
            })?;
            let mut output = Vec::with_capacity(data.len());
            decoder.read_to_end(&mut output).map_err(|e| {
                SpectreError::Compression(format!("Zstd decompression failed: {}", e))
            })?;
            output
        }
        CompressionType::None => data.to_vec(),
    };

    Ok(Decompressed {
        data: decompressed,
        wire_size,
        compression,
    })
}

/// Trait for types that can be decompressed
pub trait Decompress {
    /// Decompress the data if it's compressed
    fn decompress_with(&self, compression: CompressionType) -> Result<Decompressed, SpectreError>;

    /// Auto-detect compression from Content-Encoding header and decompress
    fn decompress_auto(&self, content_encoding: Option<&str>)
        -> Result<Decompressed, SpectreError>;
}

impl Decompress for [u8] {
    fn decompress_with(&self, compression: CompressionType) -> Result<Decompressed, SpectreError> {
        decompress(self, compression)
    }

    fn decompress_auto(
        &self,
        content_encoding: Option<&str>,
    ) -> Result<Decompressed, SpectreError> {
        let compression = content_encoding
            .and_then(CompressionType::from_encoding)
            .unwrap_or(CompressionType::None);
        self.decompress_with(compression)
    }
}

impl Decompress for Vec<u8> {
    fn decompress_with(&self, compression: CompressionType) -> Result<Decompressed, SpectreError> {
        decompress(self, compression)
    }

    fn decompress_auto(
        &self,
        content_encoding: Option<&str>,
    ) -> Result<Decompressed, SpectreError> {
        let compression = content_encoding
            .and_then(CompressionType::from_encoding)
            .unwrap_or(CompressionType::None);
        self.decompress_with(compression)
    }
}

/// Parse the Accept-Encoding header value and return supported encodings
pub fn parse_accept_encoding(accept_encoding: &str) -> Vec<CompressionType> {
    let mut encodings = Vec::new();

    for part in accept_encoding.split(',') {
        let part = part.trim();
        if let Some(ct) = CompressionType::from_encoding(part) {
            encodings.push(ct);
        }
    }

    // Always add identity as fallback
    if !encodings.contains(&CompressionType::None) {
        encodings.push(CompressionType::None);
    }

    encodings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_type_from_encoding() {
        assert_eq!(
            CompressionType::from_encoding("br"),
            Some(CompressionType::Brotli)
        );
        assert_eq!(
            CompressionType::from_encoding("gzip"),
            Some(CompressionType::Gzip)
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
        assert_eq!(CompressionType::from_encoding("unknown"), None);
    }

    #[test]
    fn test_decompress_none() {
        let data = b"hello world";
        let result = data.decompress_with(CompressionType::None).unwrap();
        assert_eq!(result.data, b"hello world");
        assert_eq!(result.wire_size, 11);
    }

    #[test]
    fn test_parse_accept_encoding() {
        let encodings = parse_accept_encoding("gzip, br, deflate");
        assert!(encodings.contains(&CompressionType::Gzip));
        assert!(encodings.contains(&CompressionType::Brotli));
        assert!(encodings.contains(&CompressionType::Deflate));
    }
}
