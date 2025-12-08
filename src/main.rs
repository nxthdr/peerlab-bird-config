mod headscale;
mod bird;
mod config;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Headscale API URL
    #[arg(long, env = "HEADSCALE_API_URL", default_value = "https://headscale.nxthdr.dev/api/v1/node")]
    headscale_api_url: String,

    /// Headscale API key
    #[arg(long, env = "HEADSCALE_API_KEY")]
    headscale_api_key: String,

    /// Output file path for BIRD configuration
    #[arg(long, env = "BIRD_CONFIG_OUTPUT", default_value = "/etc/bird/peerlab_generated.conf")]
    output_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    let args = Args::parse();

    info!("peerlab-bird-config");
    info!("Headscale API: {}", args.headscale_api_url);
    info!("Output file: {}", args.output_file.display());

    let config = config::Config {
        headscale_api_url: args.headscale_api_url,
        headscale_api_key: args.headscale_api_key,
        output_file: args.output_file,
    };

    // Fetch nodes from Headscale
    let nodes = headscale::fetch_nodes(&config.headscale_api_url, &config.headscale_api_key).await?;

    info!("Fetched {} nodes from Headscale", nodes.len());

    // Generate BIRD configuration
    let bird_config = bird::generate_config(&nodes)?;

    // Check if configuration changed
    let changed = bird::write_config_if_changed(&config.output_file, &bird_config)?;

    if changed {
        info!("Configuration file updated: {}", config.output_file.display());
    } else {
        info!("Configuration unchanged");
    }

    Ok(())
}
