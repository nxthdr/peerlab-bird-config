# Peerlab BIRD Config

Generates IP→ASN mappings for BIRD BGP configuration by fetching data from Headscale and peerlab-gateway APIs.

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
  --output-file /etc/bird/peerlab_users.conf
```

## Configuration

The service requires:
- **Headscale API**: To discover nodes and their Tailscale IPs
- **Peerlab Gateway API**: To fetch user email→ASN mappings

Default values:
- Headscale API: `https://headscale.nxthdr.dev/api/v1/node`
- Peerlab Gateway API: `https://peerlab.nxthdr.dev/service/mappings`
- Output file: `/etc/bird/peerlab_users.conf`

## Generated Output

The service generates a simple tuple mapping file:

```bird
# Auto-generated IP to ASN mapping for peerlab
# Generated at: 2025-12-21T11:10:04.516323+00:00

define USER_ASN_MAP = [
    (100.64.0.10, 65000),  # matthieu@nxthdr.dev
    (100.64.0.11, 65001),  # alice@example.com
];
```

## Integration with BIRD

Include the generated file in your main BIRD configuration:

```bird
include "/etc/bird/peerlab_users.conf";
```

The static BIRD configuration (filters, templates, protocols) should be managed separately in your infrastructure repository.

## Cron Job Setup

Run every minute to keep mappings up-to-date:

```bash
# /etc/cron.d/peerlab-bird-config
* * * * * root HEADSCALE_API_KEY=xxx PEERLAB_AGENT_KEY=yyy /usr/local/bin/peerlab-bird-config >> /var/log/peerlab-bird-config.log 2>&1
```

The service only writes the file if the content has changed (SHA256 hash comparison).

