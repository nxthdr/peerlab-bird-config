use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "providerId")]
    pub provider_id: Option<String>,
    pub provider: Option<String>,
    #[serde(rename = "profilePicUrl")]
    pub profile_pic_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    #[serde(rename = "machineKey")]
    pub machine_key: String,
    #[serde(rename = "nodeKey")]
    pub node_key: String,
    #[serde(rename = "discoKey")]
    pub disco_key: String,
    #[serde(rename = "ipAddresses")]
    pub ip_addresses: Vec<String>,
    pub name: String,
    pub user: User,
    #[serde(rename = "lastSeen")]
    pub last_seen: String,
    pub expiry: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub online: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodesResponse {
    nodes: Vec<Node>,
}

pub async fn fetch_nodes(api_url: &str, api_key: &str) -> Result<Vec<Node>> {
    debug!("Fetching nodes from Headscale API: {}", api_url);

    let client = reqwest::Client::new();
    let response = client
        .get(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send()
        .await
        .context("Failed to send request to Headscale API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Headscale API returned error status {}: {}",
            status,
            body
        );
    }

    let nodes_response: NodesResponse = response
        .json()
        .await
        .context("Failed to parse Headscale API response")?;

    debug!("Successfully fetched {} nodes", nodes_response.nodes.len());

    Ok(nodes_response.nodes)
}

impl Node {
    /// Get the IPv4 address from the Tailscale range (100.64.x.x)
    pub fn get_ipv4(&self) -> Option<String> {
        self.ip_addresses
            .iter()
            .find(|ip| ip.starts_with("100.64."))
            .cloned()
    }

    /// Get the IPv6 address
    #[allow(dead_code)]
    pub fn get_ipv6(&self) -> Option<String> {
        self.ip_addresses
            .iter()
            .find(|ip| ip.contains(':'))
            .cloned()
    }

    /// Check if this node has a valid user email (OIDC authenticated)
    pub fn has_user_email(&self) -> bool {
        self.user.email.is_some() && !self.user.email.as_ref().unwrap().is_empty()
    }
}
