//! CloudMetadata struct and core implementation.

use serde::de::DeserializeOwned;

use crate::client::MetadataClient;
use crate::error::MetadataError;
use crate::provider::CloudProvider;
use crate::providers::{aws, azure, gcp};

/// Main interface for fetching cloud instance metadata.
///
/// # Example
///
/// ```ignore
/// use cloud_metadata::{CloudMetadata, MetadataError};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyConfig {
///     db_host: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), MetadataError> {
///     let metadata = CloudMetadata::detect().await?;
///     let config: MyConfig = metadata.custom_json("config").await?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct CloudMetadata {
    provider: CloudProvider,
    client: MetadataClient,
    max_size: Option<usize>,
}

impl CloudMetadata {
    /// Auto-detect the cloud provider by probing metadata endpoints.
    ///
    /// Performs parallel probes to AWS, GCP, and Azure metadata services
    /// with a 500ms timeout. Returns the first provider that responds.
    ///
    /// # Errors
    ///
    /// Returns `MetadataError::NotDetected` if no cloud provider is detected.
    pub async fn detect() -> Result<Self, MetadataError> {
        Self::detect_with_base_url(crate::client::DEFAULT_BASE_URL).await
    }

    /// Auto-detect the cloud provider using a custom base URL.
    ///
    /// This is primarily useful for testing with mock servers.
    pub async fn detect_with_base_url(base_url: &str) -> Result<Self, MetadataError> {
        let client = MetadataClient::for_detection_with_base_url(base_url)?;

        // Run all probes concurrently and return the first successful one
        tokio::select! {
            Ok(()) = gcp::probe(&client) => {
                return Ok(Self {
                    provider: CloudProvider::Gcp,
                    client: MetadataClient::with_base_url(base_url)?,
                    max_size: None,
                });
            }
            Ok(()) = aws::probe(&client) => {
                return Ok(Self {
                    provider: CloudProvider::Aws,
                    client: MetadataClient::with_base_url(base_url)?,
                    max_size: None,
                });
            }
            Ok(()) = azure::probe(&client) => {
                return Ok(Self {
                    provider: CloudProvider::Azure,
                    client: MetadataClient::with_base_url(base_url)?,
                    max_size: None,
                });
            }
            else => {}
        }

        Err(MetadataError::NotDetected)
    }

    /// Create a CloudMetadata instance for AWS.
    pub fn aws() -> Self {
        Self {
            provider: CloudProvider::Aws,
            client: MetadataClient::default(),
            max_size: None,
        }
    }

    /// Create a CloudMetadata instance for AWS with a custom base URL.
    pub fn aws_with_base_url(base_url: &str) -> Self {
        Self {
            provider: CloudProvider::Aws,
            client: MetadataClient::with_base_url(base_url).expect("failed to create HTTP client"),
            max_size: None,
        }
    }

    /// Create a CloudMetadata instance for GCP.
    pub fn gcp() -> Self {
        Self {
            provider: CloudProvider::Gcp,
            client: MetadataClient::default(),
            max_size: None,
        }
    }

    /// Create a CloudMetadata instance for GCP with a custom base URL.
    pub fn gcp_with_base_url(base_url: &str) -> Self {
        Self {
            provider: CloudProvider::Gcp,
            client: MetadataClient::with_base_url(base_url).expect("failed to create HTTP client"),
            max_size: None,
        }
    }

    /// Create a CloudMetadata instance for Azure.
    pub fn azure() -> Self {
        Self {
            provider: CloudProvider::Azure,
            client: MetadataClient::default(),
            max_size: None,
        }
    }

    /// Create a CloudMetadata instance for Azure with a custom base URL.
    pub fn azure_with_base_url(base_url: &str) -> Self {
        Self {
            provider: CloudProvider::Azure,
            client: MetadataClient::with_base_url(base_url).expect("failed to create HTTP client"),
            max_size: None,
        }
    }

    /// Set the maximum size limit for fetched data.
    ///
    /// If the fetched data exceeds this limit, `MetadataError::TooLarge` is returned.
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    /// Get the detected cloud provider.
    pub fn provider(&self) -> CloudProvider {
        self.provider
    }

    /// Fetch custom data as raw bytes.
    ///
    /// - **AWS**: Returns user-data (key parameter is ignored)
    /// - **GCP**: Returns the instance attribute with the given key
    /// - **Azure**: Returns decoded customData (key parameter is ignored)
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be fetched or decoded.
    /// Returns `MetadataError::TooLarge` if the data exceeds the configured `max_size`.
    pub async fn custom_data(&self, key: &str) -> Result<Vec<u8>, MetadataError> {
        match self.provider {
            CloudProvider::Aws => aws::fetch_user_data(&self.client, self.max_size).await,
            CloudProvider::Gcp => {
                gcp::fetch_instance_attribute(&self.client, key, self.max_size).await
            }
            CloudProvider::Azure => azure::fetch_custom_data(&self.client, self.max_size).await,
        }
    }

    /// Fetch custom data as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns `MetadataError::Utf8` if the data is not valid UTF-8.
    pub async fn custom_text(&self, key: &str) -> Result<String, MetadataError> {
        let data = self.custom_data(key).await?;
        String::from_utf8(data).map_err(|_| MetadataError::Utf8)
    }

    /// Fetch custom data and deserialize as JSON.
    ///
    /// # Errors
    ///
    /// Returns `MetadataError::Json` if deserialization fails.
    pub async fn custom_json<T: DeserializeOwned>(&self, key: &str) -> Result<T, MetadataError> {
        let data = self.custom_data(key).await?;
        serde_json::from_slice(&data).map_err(MetadataError::from)
    }

    /// Fetch a GCP project-level attribute.
    ///
    /// This method is only supported on GCP. On other providers, it returns
    /// `MetadataError::NotSupported`.
    ///
    /// # Errors
    ///
    /// Returns an error if the attribute cannot be fetched or if called on
    /// a non-GCP provider.
    pub async fn project_attribute(&self, key: &str) -> Result<String, MetadataError> {
        match self.provider {
            CloudProvider::Gcp => gcp::fetch_project_attribute(&self.client, key).await,
            _ => Err(MetadataError::NotSupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_constructor() {
        let metadata = CloudMetadata::aws();
        assert_eq!(metadata.provider(), CloudProvider::Aws);
    }

    #[test]
    fn test_gcp_constructor() {
        let metadata = CloudMetadata::gcp();
        assert_eq!(metadata.provider(), CloudProvider::Gcp);
    }

    #[test]
    fn test_azure_constructor() {
        let metadata = CloudMetadata::azure();
        assert_eq!(metadata.provider(), CloudProvider::Azure);
    }

    #[test]
    fn test_aws_with_base_url() {
        let metadata = CloudMetadata::aws_with_base_url("http://localhost:8080");
        assert_eq!(metadata.provider(), CloudProvider::Aws);
    }

    #[test]
    fn test_gcp_with_base_url() {
        let metadata = CloudMetadata::gcp_with_base_url("http://localhost:8080");
        assert_eq!(metadata.provider(), CloudProvider::Gcp);
    }

    #[test]
    fn test_azure_with_base_url() {
        let metadata = CloudMetadata::azure_with_base_url("http://localhost:8080");
        assert_eq!(metadata.provider(), CloudProvider::Azure);
    }
}
