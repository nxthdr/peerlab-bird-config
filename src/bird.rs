use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::headscale::Node;

/// Generate BIRD configuration from Headscale nodes
pub fn generate_config(nodes: &[Node], email_to_asn: &HashMap<String, u32>) -> Result<String> {
    let mut config = String::new();

    // Header
    config.push_str("# Auto-generated IP to ASN mapping for peerlab\n");
    config.push_str(&format!(
        "# Generated at: {}\n",
        chrono::Utc::now().to_rfc3339()
    ));
    config.push_str("\n");

    // Filter peerlab nodes (those with user email)
    let peerlab_nodes: Vec<&Node> = nodes.iter().filter(|n| n.has_user_email()).collect();

    info!("Found {} peerlab user nodes", peerlab_nodes.len());

    // Generate IP → ASN mapping function
    config.push_str("function get_user_asn(ip remote_ip) {\n");

    for node in &peerlab_nodes {
        if let Some(ipv4) = node.get_ipv4() {
            let email = node.user.email.as_ref().unwrap();

            // Get ASN from peerlab-gateway mapping
            if let Some(&asn) = email_to_asn.get(email) {
                config.push_str(&format!(
                    "    if (remote_ip = {}) then return {};  # {}\n",
                    ipv4, asn, email
                ));
            } else {
                warn!("No ASN mapping found for user: {}", email);
            }
        }
    }

    config.push_str("    return 0;  # Unknown IP\n");
    config.push_str("}\n");

    Ok(config)
}

/// Write configuration to file only if it has changed
/// Returns true if the file was updated
pub fn write_config_if_changed(path: &Path, content: &str) -> Result<bool> {
    // Calculate hash of new content
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let new_hash = format!("{:x}", hasher.finalize());

    // Read existing file if it exists
    let existing_hash = if path.exists() {
        let existing_content =
            fs::read_to_string(path).context("Failed to read existing configuration file")?;
        let mut hasher = Sha256::new();
        hasher.update(existing_content.as_bytes());
        format!("{:x}", hasher.finalize())
    } else {
        String::new()
    };

    // Only write if content changed
    if new_hash != existing_hash {
        debug!("Configuration changed, writing to {}", path.display());

        // Write to temporary file first
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content).context("Failed to write temporary configuration file")?;

        // Atomic rename
        fs::rename(&temp_path, path).context("Failed to rename temporary configuration file")?;

        Ok(true)
    } else {
        debug!("Configuration unchanged");
        Ok(false)
    }
}
