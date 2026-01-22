//! Cloud provider enumeration.

use std::fmt;

/// Supported cloud providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloudProvider {
    /// Amazon Web Services
    Aws,
    /// Google Cloud Platform
    Gcp,
    /// Microsoft Azure
    Azure,
}

impl fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CloudProvider::Aws => write!(f, "AWS"),
            CloudProvider::Gcp => write!(f, "GCP"),
            CloudProvider::Azure => write!(f, "Azure"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_display() {
        assert_eq!(CloudProvider::Aws.to_string(), "AWS");
        assert_eq!(CloudProvider::Gcp.to_string(), "GCP");
        assert_eq!(CloudProvider::Azure.to_string(), "Azure");
    }

    #[test]
    fn test_provider_equality() {
        assert_eq!(CloudProvider::Aws, CloudProvider::Aws);
        assert_ne!(CloudProvider::Aws, CloudProvider::Gcp);
    }

    #[test]
    fn test_provider_clone() {
        let provider = CloudProvider::Azure;
        let cloned = provider;
        assert_eq!(provider, cloned);
    }
}
