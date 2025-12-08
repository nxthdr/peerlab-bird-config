# Peerlab BIRD Config

A Rust tool that generates BIRD BGP configuration from Headscale API for the nxthdr peerlab infrastructure.

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Using environment variables
export HEADSCALE_API_KEY="your-api-key"
./peerlab-bird-config

# Or with CLI arguments
./peerlab-bird-config --headscale-api-key "your-api-key"
```

## Cron Job Setup

Run every minute to keep configuration up-to-date:

```bash
# Edit crontab
sudo crontab -e

# Add this line to run every minute
* * * * * HEADSCALE_API_KEY=your-api-key /usr/local/bin/peerlab-bird-config >> /var/log/peerlab-bird-config.log 2>&1
```

Or create `/etc/cron.d/peerlab-bird-config`:

```
# Update peerlab BIRD configuration every minute
* * * * * root HEADSCALE_API_KEY=your-api-key /usr/local/bin/peerlab-bird-config >> /var/log/peerlab-bird-config.log 2>&1
```

## Generated Configuration

The service generates a BIRD configuration with:

### IP → ASN Mapping Function

```bird
function get_expected_asn(ip remote_ip) {
    if (remote_ip = 100.64.0.7) then return 64512;  # matthieu@nxthdr.dev
    if (remote_ip = 100.64.0.8) then return 64513;  # alice@example.com
    return 0;  # Unknown IP
}
```

### Import Filter with Security Checks

```bird
filter PeerlabImportFilter {
    int expected_asn;
    int actual_asn;

    actual_asn = bgp_path.last;
    expected_asn = get_expected_asn(from);

    # Reject unknown IPs
    if (expected_asn = 0) then {
        print "REJECT: Unknown/unauthorized IP ", from;
        reject;
    }

    # Verify ASN matches
    if (actual_asn != expected_asn) then {
        print "SECURITY ALERT: ASN mismatch from ", from;
        reject;
    }

    # Accept if in private ASN range
    if (actual_asn >= 64512 && actual_asn <= 65534) then {
        accept;
    }

    reject;
}
```

## Node Naming Convention

For ASN extraction to work, Headscale nodes must follow the naming pattern:

```
peerlab-{asn}
```

Examples:
- `peerlab-64512` → ASN 64512
- `peerlab-65534` → ASN 65534

## Integration with Main BIRD Config

In your main BIRD configuration (`/etc/bird/bird.conf`):

```bird
# Include auto-generated peerlab configuration
include "/etc/bird/peerlab_generated.conf";
```

