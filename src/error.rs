//! Error types for cloud metadata operations.

use thiserror::Error;

/// Errors that can occur when fetching cloud metadata.
#[derive(Debug, Error)]
pub enum MetadataError {
    /// Cloud provider could not be detected.
    #[error("not running in a cloud environment")]
    NotDetected,

    /// The requested metadata was not found.
    #[error("metadata not found")]
    NotFound,

    /// Request timed out.
    #[error("request timeout")]
    Timeout,

    /// HTTP error with status code.
    #[error("http {0}")]
    Http(u16),

    /// Base64 decoding failed (Azure customData).
    #[error("base64 decode failed")]
    Base64,

    /// Response was not valid UTF-8.
    #[error("invalid utf-8")]
    Utf8,

    /// JSON deserialization error.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP request error.
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Operation not supported for this provider.
    #[error("operation not supported for this provider")]
    NotSupported,

    /// Response exceeds maximum allowed size.
    #[error("response too large: {0} bytes exceeds limit of {1} bytes")]
    TooLarge(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            MetadataError::NotDetected.to_string(),
            "not running in a cloud environment"
        );
        assert_eq!(MetadataError::NotFound.to_string(), "metadata not found");
        assert_eq!(MetadataError::Timeout.to_string(), "request timeout");
        assert_eq!(MetadataError::Http(404).to_string(), "http 404");
        assert_eq!(MetadataError::Base64.to_string(), "base64 decode failed");
        assert_eq!(MetadataError::Utf8.to_string(), "invalid utf-8");
        assert_eq!(
            MetadataError::NotSupported.to_string(),
            "operation not supported for this provider"
        );
    }
}
