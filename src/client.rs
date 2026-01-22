//! HTTP client wrapper for metadata requests.

use std::time::Duration;

use reqwest::Client;

/// Default timeout for metadata requests.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for provider detection probes.
pub const DETECTION_TIMEOUT: Duration = Duration::from_millis(500);

/// Default metadata service base URL (link-local address).
pub const DEFAULT_BASE_URL: &str = "http://169.254.169.254";

/// HTTP client wrapper for metadata service requests.
#[derive(Debug, Clone)]
pub struct MetadataClient {
    inner: Client,
    base_url: String,
}

impl MetadataClient {
    /// Create a new metadata client with the specified timeout and base URL.
    pub fn new(timeout: Duration, base_url: &str) -> Result<Self, reqwest::Error> {
        let inner = Client::builder()
            .timeout(timeout)
            .danger_accept_invalid_certs(false)
            .build()?;
        Ok(Self {
            inner,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Create a new metadata client with the default timeout and base URL.
    pub fn with_default_timeout() -> Result<Self, reqwest::Error> {
        Self::new(DEFAULT_TIMEOUT, DEFAULT_BASE_URL)
    }

    /// Create a new metadata client with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str) -> Result<Self, reqwest::Error> {
        Self::new(DEFAULT_TIMEOUT, base_url)
    }

    /// Create a detection client with a custom base URL (for testing).
    pub fn for_detection_with_base_url(base_url: &str) -> Result<Self, reqwest::Error> {
        Self::new(DETECTION_TIMEOUT, base_url)
    }

    /// Get the underlying reqwest client.
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Default for MetadataClient {
    fn default() -> Self {
        Self::with_default_timeout().expect("failed to create HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeout() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn test_detection_timeout() {
        assert_eq!(DETECTION_TIMEOUT, Duration::from_millis(500));
    }

    #[test]
    fn test_default_base_url() {
        assert_eq!(DEFAULT_BASE_URL, "http://169.254.169.254");
    }

    #[test]
    fn test_client_creation() {
        let client = MetadataClient::with_default_timeout().unwrap();
        assert_eq!(client.base_url(), DEFAULT_BASE_URL);
    }

    #[test]
    fn test_client_custom_base_url() {
        let client = MetadataClient::with_base_url("http://localhost:8080").unwrap();
        assert_eq!(client.base_url(), "http://localhost:8080");
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let client = MetadataClient::with_base_url("http://localhost:8080/").unwrap();
        assert_eq!(client.base_url(), "http://localhost:8080");
    }
}
