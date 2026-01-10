# PeerLab BIRD Config

Generates BIRD BGP policy enforcement configuration by fetching data from Headscale and PeerLab Gateway APIs. Creates a unified policy function that validates both ASN and prefix announcements for PeerLab users.

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Using environment variables
export HEADSCALE_API_KEY="your-api-key"
export PEERLAB_AGENT_KEY="your-agent-key"
./peerlab-bird-config

# Or with CLI arguments
./peerlab-bird-config \
  --headscale-api-key "your-api-key" \
  --peerlab-agent-key "your-agent-key" \
  --output-file /etc/bird/peerlab_generated.conf
```

## Configuration

The service requires:
- **Headscale API**: To discover nodes and their Tailscale IPs
- **PeerLab Gateway API**: To fetch user email→ASN→prefix mappings

### Environment Variables

- `HEADSCALE_API_URL`: Headscale API endpoint (default: `https://headscale.nxthdr.dev/api/v1/node`)
- `HEADSCALE_API_KEY`: Headscale API authentication key (required)
- `PEERLAB_GATEWAY_URL`: PeerLab Gateway API endpoint (default: `https://peerlab.nxthdr.dev/service/mappings`)
- `PEERLAB_AGENT_KEY`: PeerLab Gateway agent authentication key (required)
- `BIRD_CONFIG_OUTPUT`: Output file path (default: `/etc/bird/peerlab_generated.conf`)

## Generated Output

The service generates a BIRD policy enforcement function that validates both ASN and prefixes:

```bird
# Auto-generated user policy for PeerLab
# Generated at: 2025-01-10T20:30:00.000000+00:00

function enforce_user_policy(ip remote_ip) {
    # User: matthieu@nxthdr.dev
    if (remote_ip = 100.64.0.10) then {
        if (bgp_path.last != 65000) then reject;
        if !(net ~ [ 2a06:de00:5b:1000::/48 ]) then reject;
        accept;
    }

    # User: alice@example.com
    if (remote_ip = 100.64.0.11) then {
        if (bgp_path.last != 65001) then reject;
        if !(net ~ [ 2a06:de00:5b:2000::/48, 2a06:de00:5b:3000::/48 ]) then reject;
        accept;
    }

    # Unknown user
    reject;
}
```

This function:
- Matches incoming BGP connections by Tailscale IP
- Validates the peer's ASN matches their assigned ASN
- Validates announced prefixes match their active leases
- Rejects unauthorized announcements

## Integration with BIRD

Include the generated file in your main BIRD configuration:

```bird
include "/etc/bird/peerlab_generated.conf";
```

Then use the `enforce_user_policy()` function in your import filter:

```bird
filter enforce_user_asn_and_prefix {
    enforce_user_policy(from);
}

protocol bgp peerlab from peerlab_template {
    ipv6 {
        import filter enforce_user_asn_and_prefix;
        export filter PeerlabExportFilter;
    };
}
```

The static BIRD configuration (filters, templates, protocols) should be managed separately in your infrastructure repository.

## Systemd Timer Setup

Run every minute to keep mappings up-to-date:

```ini
# /etc/systemd/system/peerlab-bird-config.service
[Unit]
Description=PeerLab BIRD Config Generator
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
Restart=on-failure
RestartSec=30s

ExecStart=/usr/local/bin/peerlab-bird-config
ExecStartPost=/usr/bin/sudo /usr/local/sbin/birdc configure

Environment="HEADSCALE_API_KEY=your-api-key"
Environment="PEERLAB_AGENT_KEY=your-agent-key"

[Install]
WantedBy=multi-user.target
```

```ini
# /etc/systemd/system/peerlab-bird-config.timer
[Unit]
Description=Run PeerLab BIRD Config Generator every minute

[Timer]
OnBootSec=1min
OnUnitActiveSec=1min

[Install]
WantedBy=timers.target
```

Enable and start the timer:
```bash
sudo systemctl enable --now peerlab-bird-config.timer
```

The service only writes the file if the content has changed (SHA256 hash comparison), and BIRD is automatically reconfigured when changes are detected.

