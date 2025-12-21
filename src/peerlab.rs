use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub async fn fetch_mappings(api_url: &str, agent_key: &str) -> Result<HashMap<String, u32>> {
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

    // Build email -> ASN mapping
    let mut email_to_asn = HashMap::new();
    for mapping in mappings_response.mappings {
        if let Some(email) = mapping.email {
            if !email.is_empty() {
                email_to_asn.insert(email, mapping.asn);
            }
        }
    }

    debug!(
        "Successfully fetched {} email->ASN mappings",
        email_to_asn.len()
    );

    Ok(email_to_asn)
}
