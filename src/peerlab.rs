use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserMapping {
    pub user_hash: String,
    pub user_id: String,
    pub email: Option<String>,
    pub asn: u32,
    pub prefixes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MappingsResponse {
    mappings: Vec<UserMapping>,
}

/// Fetch user ASN mappings from peerlab-gateway
pub async fn fetch_mappings(api_url: &str, agent_key: &str) -> Result<Vec<UserMapping>> {
    debug!("Fetching ASN mappings from peerlab-gateway: {}", api_url);

    let client = reqwest::Client::new();
    let response = client
        .get(api_url)
        .header("Authorization", format!("Bearer {}", agent_key))
        .header("Content-Type", "application/json")
        .send()
        .await
        .context("Failed to send request to peerlab-gateway")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Peerlab-gateway API returned error status {}: {}",
            status,
            body
        );
    }

    let mappings_response: MappingsResponse = response
        .json()
        .await
        .context("Failed to parse peerlab-gateway API response")?;

    debug!(
        "Successfully fetched {} user mappings",
        mappings_response.mappings.len()
    );

    Ok(mappings_response.mappings)
}
