//! CLI binary for cloud-metadata crate.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use cloud_metadata::{CloudMetadata, CloudProvider, MetadataError};

/// Default metadata key for GCP instance attributes.
/// AWS and Azure ignore this key.
const DEFAULT_METADATA_KEY: &str = "user-data-json";

#[derive(Parser)]
#[command(name = "cloud-metadata")]
#[command(
    author,
    version,
    about = "Fetch custom instance metadata from cloud providers"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect the current cloud provider
    Detect,

    /// Fetch custom metadata
    Fetch {
        /// The metadata key (used by GCP, ignored by AWS/Azure)
        #[arg(default_value = DEFAULT_METADATA_KEY)]
        key: String,

        /// Explicitly specify the cloud provider instead of auto-detecting
        #[arg(short, long, value_parser = parse_provider)]
        provider: Option<CloudProvider>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// Maximum size in bytes to accept (fails if exceeded)
        #[arg(short, long)]
        max_size: Option<usize>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Raw,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "raw" => Ok(OutputFormat::Raw),
            _ => Err(format!("unknown format: {}", s)),
        }
    }
}

fn parse_provider(s: &str) -> Result<CloudProvider, String> {
    match s.to_lowercase().as_str() {
        "aws" => Ok(CloudProvider::Aws),
        "gcp" => Ok(CloudProvider::Gcp),
        "azure" => Ok(CloudProvider::Azure),
        _ => Err(format!(
            "unknown provider: {} (expected aws, gcp, or azure)",
            s
        )),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), MetadataError> {
    match cli.command {
        Commands::Detect => {
            let metadata = CloudMetadata::detect().await?;
            println!("{}", metadata.provider());
            Ok(())
        }

        Commands::Fetch {
            key,
            provider,
            format,
            max_size,
        } => {
            let metadata = match provider {
                Some(CloudProvider::Aws) => CloudMetadata::aws(),
                Some(CloudProvider::Gcp) => CloudMetadata::gcp(),
                Some(CloudProvider::Azure) => CloudMetadata::azure(),
                None => CloudMetadata::detect().await?,
            };

            let metadata = match max_size {
                Some(size) => metadata.with_max_size(size),
                None => metadata,
            };

            match format {
                OutputFormat::Text => {
                    let text = metadata.custom_text(&key).await?;
                    println!("{}", text);
                }
                OutputFormat::Json => {
                    let value: serde_json::Value = metadata.custom_json(&key).await?;
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
                OutputFormat::Raw => {
                    let data = metadata.custom_data(&key).await?;
                    io::stdout().write_all(&data)?;
                }
            }
            Ok(())
        }
    }
}
