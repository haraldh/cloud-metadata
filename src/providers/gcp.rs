//! GCP metadata implementation.

use crate::client::MetadataClient;
use crate::error::MetadataError;

/// GCP metadata service base path.
const METADATA_PATH: &str = "/computeMetadata/v1";

/// Instance attributes path.
const INSTANCE_ATTRIBUTES_PATH: &str = "/computeMetadata/v1/instance/attributes";

/// Project attributes path.
const PROJECT_ATTRIBUTES_PATH: &str = "/computeMetadata/v1/project/attributes";

/// Required header for GCP metadata requests.
const METADATA_FLAVOR_HEADER: &str = "Metadata-Flavor";

/// Required header value for GCP metadata requests.
const METADATA_FLAVOR_VALUE: &str = "Google";

/// Probe GCP metadata service to check if we're running on GCP.
pub async fn probe(client: &MetadataClient) -> Result<(), MetadataError> {
    let url = format!("{}{}", client.base_url(), METADATA_PATH);

    let response = client
        .inner()
        .get(&url)
        .header(METADATA_FLAVOR_HEADER, METADATA_FLAVOR_VALUE)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(MetadataError::NotDetected)
    }
}

/// Fetch an instance attribute by key.
pub async fn fetch_instance_attribute(
    client: &MetadataClient,
    key: &str,
) -> Result<Vec<u8>, MetadataError> {
    let url = format!("{}{}/{}", client.base_url(), INSTANCE_ATTRIBUTES_PATH, key);

    let response = client
        .inner()
        .get(&url)
        .header(METADATA_FLAVOR_HEADER, METADATA_FLAVOR_VALUE)
        .send()
        .await?;

    let status = response.status();
    if status.as_u16() == 404 {
        return Err(MetadataError::NotFound);
    }
    if !status.is_success() {
        return Err(MetadataError::Http(status.as_u16()));
    }

    Ok(response.bytes().await?.to_vec())
}

/// Fetch a project attribute by key.
pub async fn fetch_project_attribute(
    client: &MetadataClient,
    key: &str,
) -> Result<String, MetadataError> {
    let url = format!("{}{}/{}", client.base_url(), PROJECT_ATTRIBUTES_PATH, key);

    let response = client
        .inner()
        .get(&url)
        .header(METADATA_FLAVOR_HEADER, METADATA_FLAVOR_VALUE)
        .send()
        .await?;

    let status = response.status();
    if status.as_u16() == 404 {
        return Err(MetadataError::NotFound);
    }
    if !status.is_success() {
        return Err(MetadataError::Http(status.as_u16()));
    }

    response.text().await.map_err(MetadataError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        assert!(INSTANCE_ATTRIBUTES_PATH.starts_with(METADATA_PATH));
        assert!(PROJECT_ATTRIBUTES_PATH.starts_with(METADATA_PATH));
    }

    #[test]
    fn test_url_construction() {
        let base = "http://localhost:8080";
        let key = "my-config";
        let url = format!("{}{}/{}", base, INSTANCE_ATTRIBUTES_PATH, key);
        assert_eq!(
            url,
            "http://localhost:8080/computeMetadata/v1/instance/attributes/my-config"
        );
    }
}
