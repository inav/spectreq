//! Streaming downloads support for Spectre client
//!
//! This module provides streaming response capabilities, allowing you to
//! download large files in chunks without loading the entire response into memory.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

use crate::core::SpectreError;
use bytes::Bytes;

/// Streaming response for chunked reading
pub struct StreamingResponse {
    status: u16,
    headers: Vec<(String, String)>,
    reader: Box<dyn AsyncRead + Send + Unpin>,
    #[allow(dead_code)]
    content_encoding: Option<String>, // Reserved for future decompression support
    #[allow(dead_code)]
    decompressed: bool, // Reserved for future decompression support
    total_read: usize,
    wire_size: usize,
}

impl StreamingResponse {
    /// Create a new streaming response
    pub fn new(
        status: u16,
        headers: Vec<(String, String)>,
        reader: Box<dyn AsyncRead + Send + Unpin>,
        wire_size: usize,
    ) -> Self {
        let content_encoding = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-encoding"))
            .and_then(|(_, v)| if v.is_empty() { None } else { Some(v.clone()) });

        Self {
            status,
            headers,
            reader,
            content_encoding,
            decompressed: false,
            total_read: 0,
            wire_size,
        }
    }

    /// Create a streaming response without decompression
    pub fn new_raw(
        status: u16,
        headers: Vec<(String, String)>,
        reader: Box<dyn AsyncRead + Send + Unpin>,
        wire_size: usize,
    ) -> Self {
        Self {
            status,
            headers,
            reader,
            content_encoding: None,
            decompressed: false,
            total_read: 0,
            wire_size,
        }
    }

    /// Get the HTTP status code
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get the response headers
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// Get a specific header value
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }

    /// Get the content type
    pub fn content_type(&self) -> Option<&String> {
        self.header("content-type")
    }

    /// Get the content length if available
    pub fn content_length(&self) -> Option<usize> {
        self.header("content-length").and_then(|v| v.parse().ok())
    }

    /// Get the wire size (compressed size from network)
    pub fn wire_size(&self) -> usize {
        self.wire_size
    }

    /// Get the total bytes read so far
    pub fn total_read(&self) -> usize {
        self.total_read
    }

    /// Check if request was successful
    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Read a chunk of data from the response
    ///
    /// This reads up to `buf.len()` bytes into the buffer.
    /// Returns the number of bytes read.
    pub async fn read_chunk(&mut self, buf: &mut [u8]) -> Result<usize, SpectreError> {
        use tokio::io::AsyncReadExt;
        let n = self.reader.read(buf).await?;
        self.total_read += n;
        Ok(n)
    }

    /// Read all remaining data until end of stream
    pub async fn read_all(&mut self) -> Result<Vec<u8>, SpectreError> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];

        loop {
            let n = self.read_chunk(&mut chunk).await?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..n]);
        }

        Ok(buffer)
    }

    /// Stream response to a file
    pub async fn stream_to_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<usize, SpectreError> {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(path).await?;

        let mut total = 0;
        let mut chunk = [0u8; 8192];

        loop {
            let n = self.read_chunk(&mut chunk).await?;
            if n == 0 {
                break;
            }

            file.write_all(&chunk[..n]).await?;
            total += n;
        }

        file.flush().await?;

        Ok(total)
    }

    /// Stream response with progress callback
    pub async fn stream_with_progress<F>(
        &mut self,
        mut progress: F,
    ) -> Result<Vec<u8>, SpectreError>
    where
        F: FnMut(usize, Option<usize>), // (bytes_read, total_size)
    {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];

        let content_length = self.content_length();

        loop {
            let n = self.read_chunk(&mut chunk).await?;
            if n == 0 {
                break;
            }

            buffer.extend_from_slice(&chunk[..n]);
            progress(self.total_read, content_length);
        }

        Ok(buffer)
    }
}

/// AsyncRead adapter for a byte slice
///
/// This type wraps a `Bytes` object and implements `AsyncRead`,
/// allowing it to be used as a streaming data source.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::client::streaming::SliceReader;
/// use bytes::Bytes;
///
/// let data = Bytes::from("Hello, World!");
/// let reader = SliceReader::new(data);
/// // Use reader with streaming APIs...
/// ```
pub struct SliceReader {
    data: Bytes,
    pos: usize,
}

impl SliceReader {
    /// Create a new slice reader
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to wrap
    pub fn new(data: Bytes) -> Self {
        Self { data, pos: 0 }
    }

    /// Get the number of remaining bytes to read
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Check if all bytes have been read
    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }
}

impl AsyncRead for SliceReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.is_empty() {
            return Poll::Ready(Ok(()));
        }

        let remaining = self.remaining();
        let to_fill = buf.remaining().min(remaining);

        // Get a slice of our data starting at current position
        let data_slice = &self.data[self.pos..self.pos + to_fill];

        // Fill the buffer
        let _filled = buf.filled().len();
        buf.put_slice(data_slice);

        // Update position
        self.pos += to_fill;

        Poll::Ready(Ok(()))
    }
}

/// Create a streaming response from bytes
///
/// # Arguments
///
/// * `status` - HTTP status code
/// * `headers` - Response headers
/// * `data` - The response body bytes
/// * `wire_size` - The compressed size from the network
///
/// # Examples
///
/// ```rust,ignore
/// use crate::client::streaming::streaming_response_from_bytes;
/// use bytes::Bytes;
///
/// let data = Bytes::from("Hello, World!");
/// let response = streaming_response_from_bytes(200, vec![], data.clone(), 13);
/// ```
pub fn streaming_response_from_bytes(
    status: u16,
    headers: Vec<(String, String)>,
    data: Bytes,
    wire_size: usize,
) -> StreamingResponse {
    let reader = Box::new(SliceReader::new(data)) as Box<dyn AsyncRead + Send + Unpin>;
    StreamingResponse::new_raw(status, headers, reader, wire_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slice_reader() {
        let data = Bytes::from("Hello, World!");
        let reader = SliceReader::new(data.clone());

        assert!(!reader.is_empty());
        assert_eq!(reader.remaining(), 13);
    }

    #[tokio::test]
    async fn test_streaming_response_from_bytes() {
        let data = Bytes::from("Hello, World!");
        let headers = vec![("content-type".to_string(), "text/plain".to_string())];

        let mut response = streaming_response_from_bytes(200, headers, data.clone(), 13);

        assert_eq!(response.status(), 200);
        assert!(response.ok());
        assert_eq!(response.content_length(), None);

        let result = response.read_all().await.unwrap();
        assert_eq!(result, b"Hello, World!".to_vec());
    }

    #[tokio::test]
    async fn test_streaming_response_read_chunk() {
        let data = Bytes::from("Hello, World!");
        let headers = vec![];

        let mut response = streaming_response_from_bytes(200, headers, data.clone(), 13);

        let mut buf = [0u8; 5];
        let n = response.read_chunk(&mut buf).await.unwrap();

        assert_eq!(n, 5);
        assert_eq!(&buf, b"Hello");
        assert_eq!(response.total_read(), 5);

        let remaining = response.read_all().await.unwrap();
        assert_eq!(remaining, b", World!");
    }

    #[test]
    fn test_streaming_response_new() {
        let reader = Box::new(SliceReader::new(Bytes::new())) as Box<dyn AsyncRead + Send + Unpin>;
        let response = StreamingResponse::new(200, vec![], reader, 0);
        assert_eq!(response.status(), 200);
        assert!(response.ok());
    }
}
