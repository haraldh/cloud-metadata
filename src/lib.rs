//! Minimal Rust crate for fetching custom instance metadata from AWS, GCP, and Azure VMs.
//!
//! This crate provides a simple API for fetching custom metadata (user-data, instance
//! attributes, or customData) from cloud provider metadata services.
//!
//! # Features
//!
//! - Auto-detect cloud provider
//! - Fetch custom metadata as bytes, string, or JSON
//! - Support for AWS IMDSv2, GCP Metadata Server, and Azure IMDS
//! - Automatic base64 decoding for Azure customData
//!
//! # Example
//!
//! ```ignore
//! use cloud_metadata::{CloudMetadata, MetadataError};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct MyConfig {
//!     db_host: String,
//!     feature_flags: Vec<String>,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), MetadataError> {
//!     // Auto-detect cloud provider
//!     let metadata = CloudMetadata::detect().await?;
//!
//!     // Fetch and deserialize JSON config
//!     let config: MyConfig = metadata.custom_json("config").await?;
//!
//!     // Or fetch raw bytes
//!     let raw: Vec<u8> = metadata.custom_data("config").await?;
//!
//!     // Or fetch as string
//!     let text: String = metadata.custom_text("config").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Provider-Specific Behavior
//!
//! | Provider | Metadata Source | Key Parameter |
//! |----------|-----------------|---------------|
//! | AWS | User-data | Ignored |
//! | GCP | Instance attribute | Used as attribute name |
//! | Azure | customData (base64 decoded) | Ignored |

mod client;
mod error;
mod metadata;
mod provider;
mod providers;

pub use error::MetadataError;
pub use metadata::CloudMetadata;
pub use provider::CloudProvider;
