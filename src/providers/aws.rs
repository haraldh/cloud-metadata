//! AWS IMDSv2 metadata implementation.

use crate::client::MetadataClient;
use crate::error::MetadataError;

/// AWS IMDSv2 token endpoint path.
const TOKEN_PATH: &str = "/latest/api/token";

/// AWS user-data endpoint path.
const USER_DATA_PATH: &str = "/latest/user-data";

/// Token TTL header name.
const TOKEN_TTL_HEADER: &str = "X-aws-ec2-metadata-token-ttl-seconds";

/// Token header name for requests.
const TOKEN_HEADER: &str = "X-aws-ec2-metadata-token";

/// Probe AWS metadata service to check if we're running on AWS.
pub async fn probe(client: &MetadataClient) -> Result<(), MetadataError> {
    let url = format!("{}{}", client.base_url(), TOKEN_PATH);

    let response = client
        .inner()
        .put(&url)
        .header(TOKEN_TTL_HEADER, "1")
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(MetadataError::NotDetected)
    }
}

/// Get an IMDSv2 token.
async fn get_token(client: &MetadataClient) -> Result<String, MetadataError> {
    let url = format!("{}{}", client.base_url(), TOKEN_PATH);

    let response = client
        .inner()
        .put(&url)
        .header(TOKEN_TTL_HEADER, "60")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(MetadataError::Http(response.status().as_u16()));
    }

    response.text().await.map_err(MetadataError::from)
}

/// Fetch user-data from AWS metadata service.
pub async fn fetch_user_data(client: &MetadataClient) -> Result<Vec<u8>, MetadataError> {
    let token = get_token(client).await?;
    let url = format!("{}{}", client.base_url(), USER_DATA_PATH);

    let response = client
        .inner()
        .get(&url)
        .header(TOKEN_HEADER, &token)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        assert_eq!(TOKEN_PATH, "/latest/api/token");
        assert_eq!(USER_DATA_PATH, "/latest/user-data");
    }
}
