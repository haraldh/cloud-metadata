//! Azure metadata implementation with base64 decoding.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::client::MetadataClient;
use crate::error::MetadataError;

/// Azure metadata service base path.
const METADATA_PATH: &str = "/metadata";

/// Azure customData endpoint path.
const CUSTOM_DATA_PATH: &str = "/metadata/instance/compute/customData";

/// API version query parameter.
const API_VERSION: &str = "2021-02-01";

/// Required header for Azure metadata requests.
const METADATA_HEADER: &str = "Metadata";

/// Required header value for Azure metadata requests.
const METADATA_VALUE: &str = "true";

/// Probe Azure metadata service to check if we're running on Azure.
pub async fn probe(client: &MetadataClient) -> Result<(), MetadataError> {
    let url = format!(
        "{}{}?api-version={}",
        client.base_url(),
        METADATA_PATH,
        API_VERSION
    );

    let response = client
        .inner()
        .get(&url)
        .header(METADATA_HEADER, METADATA_VALUE)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(MetadataError::NotDetected)
    }
}

/// Fetch customData from Azure metadata service.
/// Azure returns base64-encoded data, which is automatically decoded.
pub async fn fetch_custom_data(client: &MetadataClient) -> Result<Vec<u8>, MetadataError> {
    let url = format!("{}{}", client.base_url(), CUSTOM_DATA_PATH);

    let response = client
        .inner()
        .get(&url)
        .query(&[("api-version", API_VERSION), ("format", "text")])
        .header(METADATA_HEADER, METADATA_VALUE)
        .send()
        .await?;

    let status = response.status();
    if status.as_u16() == 404 {
        return Err(MetadataError::NotFound);
    }
    if !status.is_success() {
        return Err(MetadataError::Http(status.as_u16()));
    }

    let b64 = response.text().await?;

    // Handle empty response
    if b64.is_empty() {
        return Err(MetadataError::NotFound);
    }

    STANDARD.decode(&b64).map_err(|_| MetadataError::Base64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        assert!(CUSTOM_DATA_PATH.starts_with(METADATA_PATH));
        assert_eq!(API_VERSION, "2021-02-01");
    }

    #[test]
    fn test_base64_decode() {
        let encoded = STANDARD.encode(b"hello world");
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(decoded, b"hello world");
    }

    #[test]
    fn test_base64_decode_json() {
        let json = r#"{"key": "value"}"#;
        let encoded = STANDARD.encode(json);
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(decoded, json.as_bytes());
    }
}
