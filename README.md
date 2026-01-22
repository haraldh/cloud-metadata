# cloud-metadata

Minimal Rust crate for fetching custom instance metadata from AWS, GCP, and Azure VMs.

## Features

- Auto-detect cloud provider
- Fetch custom metadata as bytes, string, or JSON
- Support for AWS IMDSv2, GCP Metadata Server, and Azure IMDS
- Automatic base64 decoding for Azure customData
- No OpenSSL dependency (uses rustls)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cloud-metadata = "0.1"
```

Or install the CLI:

```bash
cargo install cloud-metadata
```

## Library Usage

```rust
use cloud_metadata::{CloudMetadata, MetadataError};
use serde::Deserialize;

#[derive(Deserialize)]
struct MyConfig {
    db_host: String,
    feature_flags: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), MetadataError> {
    // Auto-detect cloud provider
    let metadata = CloudMetadata::detect().await?;

    // Fetch and deserialize JSON config
    let config: MyConfig = metadata.custom_json("user-data-json").await?;

    // Or fetch as raw bytes
    let raw: Vec<u8> = metadata.custom_data("user-data-json").await?;

    // Or fetch as string
    let text: String = metadata.custom_text("user-data-json").await?;

    Ok(())
}
```

### Explicit Provider

```rust
// Skip auto-detection if you know the provider
let metadata = CloudMetadata::aws();
let metadata = CloudMetadata::gcp();
let metadata = CloudMetadata::azure();
```

### GCP Project Attributes

```rust
let metadata = CloudMetadata::gcp();
let value = metadata.project_attribute("my-project-key").await?;
```

## CLI Usage

```bash
# Auto-detect provider and fetch metadata
cloud-metadata fetch

# Fetch with explicit provider
cloud-metadata fetch --provider gcp

# Fetch specific key (for GCP)
cloud-metadata fetch my-custom-key

# Output as JSON
cloud-metadata fetch --format json

# Detect provider only
cloud-metadata detect
```

## Provider-Specific Behavior

| Provider | Metadata Source | Key Parameter | Encoding |
|----------|-----------------|---------------|----------|
| AWS | user-data | Ignored | Raw |
| GCP | instance/attributes/{key} | Required | Raw |
| Azure | customData | Ignored | Base64 (auto-decoded) |

## Instance Configuration Examples

### AWS (Terraform)

```hcl
resource "aws_instance" "example" {
  user_data = jsonencode({
    db_host       = "postgres.internal"
    feature_flags = ["new_ui"]
  })
}
```

### GCP (Terraform)

```hcl
resource "google_compute_instance" "example" {
  metadata = {
    user-data-json = jsonencode({
      db_host       = "postgres.internal"
      feature_flags = ["new_ui"]
    })
  }
}
```

### Azure (Terraform)

```hcl
resource "azurerm_linux_virtual_machine" "example" {
  custom_data = base64encode(jsonencode({
    db_host       = "postgres.internal"
    feature_flags = ["new_ui"]
  }))
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
