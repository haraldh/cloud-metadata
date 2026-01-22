//! Integration tests using wiremock to simulate cloud metadata services.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Deserialize;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use cloud_metadata::{CloudMetadata, CloudProvider, MetadataError};

/// Test configuration struct for JSON deserialization tests.
#[derive(Debug, Deserialize, PartialEq)]
struct TestConfig {
    db_host: String,
    port: u16,
}

// =============================================================================
// AWS Tests
// =============================================================================

mod aws {
    use super::*;

    async fn setup_aws_mock(server: &MockServer, user_data: &str) {
        // Mock the token endpoint
        Mock::given(method("PUT"))
            .and(path("/latest/api/token"))
            .and(header("X-aws-ec2-metadata-token-ttl-seconds", "60"))
            .respond_with(ResponseTemplate::new(200).set_body_string("mock-token"))
            .mount(server)
            .await;

        // Mock the user-data endpoint
        Mock::given(method("GET"))
            .and(path("/latest/user-data"))
            .and(header("X-aws-ec2-metadata-token", "mock-token"))
            .respond_with(ResponseTemplate::new(200).set_body_string(user_data))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn test_aws_fetch_user_data() {
        let server = MockServer::start().await;
        let user_data = r#"{"db_host": "postgres.internal", "port": 5432}"#;
        setup_aws_mock(&server, user_data).await;

        let metadata = CloudMetadata::aws_with_base_url(&server.uri());
        let data = metadata.custom_data("ignored").await.unwrap();

        assert_eq!(String::from_utf8(data).unwrap(), user_data);
    }

    #[tokio::test]
    async fn test_aws_fetch_user_data_as_text() {
        let server = MockServer::start().await;
        let user_data = "Hello, AWS!";
        setup_aws_mock(&server, user_data).await;

        let metadata = CloudMetadata::aws_with_base_url(&server.uri());
        let text = metadata.custom_text("ignored").await.unwrap();

        assert_eq!(text, user_data);
    }

    #[tokio::test]
    async fn test_aws_fetch_user_data_as_json() {
        let server = MockServer::start().await;
        let user_data = r#"{"db_host": "postgres.internal", "port": 5432}"#;
        setup_aws_mock(&server, user_data).await;

        let metadata = CloudMetadata::aws_with_base_url(&server.uri());
        let config: TestConfig = metadata.custom_json("ignored").await.unwrap();

        assert_eq!(
            config,
            TestConfig {
                db_host: "postgres.internal".to_string(),
                port: 5432,
            }
        );
    }

    #[tokio::test]
    async fn test_aws_user_data_not_found() {
        let server = MockServer::start().await;

        // Mock the token endpoint
        Mock::given(method("PUT"))
            .and(path("/latest/api/token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("mock-token"))
            .mount(&server)
            .await;

        // Mock 404 for user-data
        Mock::given(method("GET"))
            .and(path("/latest/user-data"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::aws_with_base_url(&server.uri());
        let result = metadata.custom_data("ignored").await;

        assert!(matches!(result, Err(MetadataError::NotFound)));
    }

    #[tokio::test]
    async fn test_aws_provider() {
        let metadata = CloudMetadata::aws_with_base_url("http://localhost:1234");
        assert_eq!(metadata.provider(), CloudProvider::Aws);
    }
}

// =============================================================================
// GCP Tests
// =============================================================================

mod gcp {
    use super::*;

    async fn setup_gcp_mock(server: &MockServer, key: &str, value: &str) {
        Mock::given(method("GET"))
            .and(path(format!(
                "/computeMetadata/v1/instance/attributes/{}",
                key
            )))
            .and(header("Metadata-Flavor", "Google"))
            .respond_with(ResponseTemplate::new(200).set_body_string(value))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn test_gcp_fetch_instance_attribute() {
        let server = MockServer::start().await;
        let config = r#"{"db_host": "postgres.internal", "port": 5432}"#;
        setup_gcp_mock(&server, "config", config).await;

        let metadata = CloudMetadata::gcp_with_base_url(&server.uri());
        let data = metadata.custom_data("config").await.unwrap();

        assert_eq!(String::from_utf8(data).unwrap(), config);
    }

    #[tokio::test]
    async fn test_gcp_fetch_as_text() {
        let server = MockServer::start().await;
        setup_gcp_mock(&server, "my-key", "my-value").await;

        let metadata = CloudMetadata::gcp_with_base_url(&server.uri());
        let text = metadata.custom_text("my-key").await.unwrap();

        assert_eq!(text, "my-value");
    }

    #[tokio::test]
    async fn test_gcp_fetch_as_json() {
        let server = MockServer::start().await;
        let config = r#"{"db_host": "postgres.internal", "port": 5432}"#;
        setup_gcp_mock(&server, "config", config).await;

        let metadata = CloudMetadata::gcp_with_base_url(&server.uri());
        let config: TestConfig = metadata.custom_json("config").await.unwrap();

        assert_eq!(
            config,
            TestConfig {
                db_host: "postgres.internal".to_string(),
                port: 5432,
            }
        );
    }

    #[tokio::test]
    async fn test_gcp_attribute_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/computeMetadata/v1/instance/attributes/missing"))
            .and(header("Metadata-Flavor", "Google"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::gcp_with_base_url(&server.uri());
        let result = metadata.custom_data("missing").await;

        assert!(matches!(result, Err(MetadataError::NotFound)));
    }

    #[tokio::test]
    async fn test_gcp_project_attribute() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(
                "/computeMetadata/v1/project/attributes/project-config",
            ))
            .and(header("Metadata-Flavor", "Google"))
            .respond_with(ResponseTemplate::new(200).set_body_string("project-value"))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::gcp_with_base_url(&server.uri());
        let value = metadata.project_attribute("project-config").await.unwrap();

        assert_eq!(value, "project-value");
    }

    #[tokio::test]
    async fn test_gcp_provider() {
        let metadata = CloudMetadata::gcp_with_base_url("http://localhost:1234");
        assert_eq!(metadata.provider(), CloudProvider::Gcp);
    }
}

// =============================================================================
// Azure Tests
// =============================================================================

mod azure {
    use super::*;

    async fn setup_azure_mock(server: &MockServer, custom_data: &[u8]) {
        let encoded = STANDARD.encode(custom_data);

        Mock::given(method("GET"))
            .and(path("/metadata/instance/compute/customData"))
            .and(query_param("api-version", "2021-02-01"))
            .and(query_param("format", "text"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_string(encoded))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn test_azure_fetch_custom_data() {
        let server = MockServer::start().await;
        let data = b"Hello, Azure!";
        setup_azure_mock(&server, data).await;

        let metadata = CloudMetadata::azure_with_base_url(&server.uri());
        let result = metadata.custom_data("ignored").await.unwrap();

        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn test_azure_fetch_as_text() {
        let server = MockServer::start().await;
        let data = "Hello, Azure!";
        setup_azure_mock(&server, data.as_bytes()).await;

        let metadata = CloudMetadata::azure_with_base_url(&server.uri());
        let text = metadata.custom_text("ignored").await.unwrap();

        assert_eq!(text, data);
    }

    #[tokio::test]
    async fn test_azure_fetch_as_json() {
        let server = MockServer::start().await;
        let json = r#"{"db_host": "postgres.internal", "port": 5432}"#;
        setup_azure_mock(&server, json.as_bytes()).await;

        let metadata = CloudMetadata::azure_with_base_url(&server.uri());
        let config: TestConfig = metadata.custom_json("ignored").await.unwrap();

        assert_eq!(
            config,
            TestConfig {
                db_host: "postgres.internal".to_string(),
                port: 5432,
            }
        );
    }

    #[tokio::test]
    async fn test_azure_base64_decodes_automatically() {
        let server = MockServer::start().await;
        let original = b"binary\x00data\xff";
        setup_azure_mock(&server, original).await;

        let metadata = CloudMetadata::azure_with_base_url(&server.uri());
        let result = metadata.custom_data("ignored").await.unwrap();

        assert_eq!(result, original);
    }

    #[tokio::test]
    async fn test_azure_custom_data_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/metadata/instance/compute/customData"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::azure_with_base_url(&server.uri());
        let result = metadata.custom_data("ignored").await;

        assert!(matches!(result, Err(MetadataError::NotFound)));
    }

    #[tokio::test]
    async fn test_azure_invalid_base64() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/metadata/instance/compute/customData"))
            .and(query_param("api-version", "2021-02-01"))
            .and(query_param("format", "text"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not-valid-base64!!!"))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::azure_with_base_url(&server.uri());
        let result = metadata.custom_data("ignored").await;

        assert!(matches!(result, Err(MetadataError::Base64)));
    }

    #[tokio::test]
    async fn test_azure_provider() {
        let metadata = CloudMetadata::azure_with_base_url("http://localhost:1234");
        assert_eq!(metadata.provider(), CloudProvider::Azure);
    }
}

// =============================================================================
// Detection Tests
// =============================================================================

mod detection {
    use super::*;

    #[tokio::test]
    async fn test_detect_aws() {
        let server = MockServer::start().await;

        // AWS probe endpoint (token endpoint with TTL=1)
        Mock::given(method("PUT"))
            .and(path("/latest/api/token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("token"))
            .mount(&server)
            .await;

        // Also need the fetch endpoints for after detection
        Mock::given(method("PUT"))
            .and(path("/latest/api/token"))
            .and(header("X-aws-ec2-metadata-token-ttl-seconds", "60"))
            .respond_with(ResponseTemplate::new(200).set_body_string("mock-token"))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::detect_with_base_url(&server.uri())
            .await
            .unwrap();
        assert_eq!(metadata.provider(), CloudProvider::Aws);
    }

    #[tokio::test]
    async fn test_detect_gcp() {
        let server = MockServer::start().await;

        // GCP probe endpoint
        Mock::given(method("GET"))
            .and(path("/computeMetadata/v1"))
            .and(header("Metadata-Flavor", "Google"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::detect_with_base_url(&server.uri())
            .await
            .unwrap();
        assert_eq!(metadata.provider(), CloudProvider::Gcp);
    }

    #[tokio::test]
    async fn test_detect_azure() {
        let server = MockServer::start().await;

        // Azure probe endpoint
        Mock::given(method("GET"))
            .and(path("/metadata"))
            .and(query_param("api-version", "2021-02-01"))
            .and(header("Metadata", "true"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let metadata = CloudMetadata::detect_with_base_url(&server.uri())
            .await
            .unwrap();
        assert_eq!(metadata.provider(), CloudProvider::Azure);
    }

    #[tokio::test]
    async fn test_detect_not_in_cloud() {
        let server = MockServer::start().await;
        // No mocks set up - all probes will fail

        let result = CloudMetadata::detect_with_base_url(&server.uri()).await;
        assert!(matches!(result, Err(MetadataError::NotDetected)));
    }
}

// =============================================================================
// Cross-Provider Tests
// =============================================================================

mod cross_provider {
    use super::*;

    #[tokio::test]
    async fn test_project_attribute_not_supported_on_aws() {
        let metadata = CloudMetadata::aws_with_base_url("http://localhost:1234");
        let result = metadata.project_attribute("key").await;
        assert!(matches!(result, Err(MetadataError::NotSupported)));
    }

    #[tokio::test]
    async fn test_project_attribute_not_supported_on_azure() {
        let metadata = CloudMetadata::azure_with_base_url("http://localhost:1234");
        let result = metadata.project_attribute("key").await;
        assert!(matches!(result, Err(MetadataError::NotSupported)));
    }
}
