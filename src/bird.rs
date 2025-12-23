use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::headscale::Node;
use crate::peerlab::UserMapping;

/// Generate BIRD configuration from Headscale nodes
pub fn generate_config(nodes: &[Node], mappings: &[UserMapping]) -> Result<String> {
    let mut config = String::new();

    // Header
    config.push_str("# Auto-generated user policy for PeerLab\n");
    config.push_str(&format!(
        "# Generated at: {}\n",
        chrono::Utc::now().to_rfc3339()
    ));
    config.push_str("\n");

    // Build email -> mapping lookup
    let email_to_mapping: HashMap<String, &UserMapping> = mappings
        .iter()
        .filter_map(|m| m.email.as_ref().map(|e| (e.clone(), m)))
        .collect();

    // Filter peerlab nodes (those with user email)
    let peerlab_nodes: Vec<&Node> = nodes.iter().filter(|n| n.has_user_email()).collect();

    info!("Found {} peerlab user nodes", peerlab_nodes.len());

    // Generate unified policy enforcement function
    config.push_str("function enforce_user_policy(ip remote_ip) {\n");

    for node in &peerlab_nodes {
        if let Some(ipv4) = node.get_ipv4() {
            let email = node.user.email.as_ref().unwrap();

            if let Some(mapping) = email_to_mapping.get(email) {
                config.push_str(&format!("    # User: {}\n", email));
                config.push_str(&format!("    if (remote_ip = {}) then {{\n", ipv4));

                // ASN check
                config.push_str(&format!(
                    "        if (bgp_path.last != {}) then reject;\n",
                    mapping.asn
                ));

                // Prefix check
                if !mapping.prefixes.is_empty() {
                    let prefixes_str = mapping.prefixes.join(", ");
                    config.push_str(&format!(
                        "        if !(net ~ [ {} ]) then reject;\n",
                        prefixes_str
                    ));
                } else {
                    warn!("No prefixes found for user: {}", email);
                    config.push_str("        reject;  # No authorized prefixes\n");
                }

                config.push_str("        accept;\n");
                config.push_str("    }\n\n");
            } else {
                warn!("No ASN mapping found for user: {}", email);
            }
        }
    }

    config.push_str("    # Unknown user\n");
    config.push_str("    reject;\n");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headscale::{Node, User};
    use crate::peerlab::UserMapping;

    #[test]
    fn test_generate_config_with_prefixes() {
        // Create test nodes
        let nodes = vec![
            Node {
                id: "1".to_string(),
                machine_key: "key1".to_string(),
                node_key: "nkey1".to_string(),
                disco_key: "dkey1".to_string(),
                ip_addresses: vec!["100.64.0.1".to_string()],
                name: "node1".to_string(),
                user: User {
                    id: "u1".to_string(),
                    name: "user1".to_string(),
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    display_name: Some("User One".to_string()),
                    email: Some("user1@example.com".to_string()),
                    provider_id: None,
                    provider: None,
                    profile_pic_url: None,
                },
                last_seen: "2024-01-01T00:00:00Z".to_string(),
                expiry: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                online: true,
            },
            Node {
                id: "2".to_string(),
                machine_key: "key2".to_string(),
                node_key: "nkey2".to_string(),
                disco_key: "dkey2".to_string(),
                ip_addresses: vec!["100.64.0.2".to_string()],
                name: "node2".to_string(),
                user: User {
                    id: "u2".to_string(),
                    name: "user2".to_string(),
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                    display_name: Some("User Two".to_string()),
                    email: Some("user2@example.com".to_string()),
                    provider_id: None,
                    provider: None,
                    profile_pic_url: None,
                },
                last_seen: "2024-01-01T00:00:00Z".to_string(),
                expiry: None,
                created_at: "2024-01-01T00:00:00Z".to_string(),
                online: true,
            },
        ];

        // Create test mappings
        let mappings = vec![
            UserMapping {
                user_hash: "hash1".to_string(),
                user_id: "u1".to_string(),
                email: Some("user1@example.com".to_string()),
                asn: 65001,
                prefixes: vec!["2001:db8:1::/48".to_string(), "2001:db8:2::/48".to_string()],
            },
            UserMapping {
                user_hash: "hash2".to_string(),
                user_id: "u2".to_string(),
                email: Some("user2@example.com".to_string()),
                asn: 65002,
                prefixes: vec!["2001:db8:3::/48".to_string()],
            },
        ];

        let config = generate_config(&nodes, &mappings).unwrap();

        // Verify unified policy function exists
        assert!(config.contains("function enforce_user_policy(ip remote_ip)"));

        // Verify user 1 policy
        assert!(config.contains("# User: user1@example.com"));
        assert!(config.contains("if (remote_ip = 100.64.0.1) then {"));
        assert!(config.contains("if (bgp_path.last != 65001) then reject;"));
        assert!(config.contains("if !(net ~ [ 2001:db8:1::/48, 2001:db8:2::/48 ]) then reject;"));

        // Verify user 2 policy
        assert!(config.contains("# User: user2@example.com"));
        assert!(config.contains("if (remote_ip = 100.64.0.2) then {"));
        assert!(config.contains("if (bgp_path.last != 65002) then reject;"));
        assert!(config.contains("if !(net ~ [ 2001:db8:3::/48 ]) then reject;"));

        // Verify accept and unknown user handling
        assert!(config.contains("accept;"));
        assert!(config.contains("# Unknown user"));
    }
}
