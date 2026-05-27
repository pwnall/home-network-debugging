# TRENDnet TEG-3102WS Configuration Utility

A Rust-based library and CLI tool for programmatically managing the TRENDnet TEG-3102WS 10-Port Multi-Gig Web Smart Switch via its REST API.

## Features

- **JWT Authentication**: Handles the switch's session-based authentication flow (via `PATCH /api/system/login`).
- **Monitoring & Status**: Retrieve system information, uptime, temperature, and firmware version.
- **System Settings**: View and update switch metadata such as Device Name, Location, and Contact.
- **Reboot Management**: Remote reboot capability.
- **L2 Management**: View the MAC address forwarding table and detailed port status/configurations (including MTU, flow control, and speed).
- **STP Management**: View and modify global Spanning Tree Protocol settings.
- **Loopback Detection (LBD)**: Enable or disable LBD globally.
- **Multicast Control**: View IGMP snooping and multicast filter settings.
- **Security Features**: Manage DHCP Snooping, Trust Ports, Jumbo Frames (via `_extended` API), and DoS protection.
- **CLI Access Control**: Enable or disable SSH and Telnet management interfaces.
- **Configuration Persistence**: Save the running configuration to the switch's flash memory.

## Prerequisites

- **Rust**: Ensure you have the Rust toolchain installed.
- **Connectivity**: The switch must be reachable over the network on port 80 (HTTP).

## Building

This crate uses `rustls-tls` to avoid dependencies on system OpenSSL libraries.

```bash
cargo build -p trendnet-teg3102ws-config
```

## CLI Usage

The binary `trendnet-config` wraps the library's functionality. By default, it looks for credentials in `data/network.json`.

### Global Options
- `-c, --config <PATH>`: Path to the network configuration JSON file (default: `data/network.json`).

### Commands

**Monitoring & System**
```bash
# Show current system status (firmware, uptime, etc.)
cargo run -p trendnet-teg3102ws-config -- status

# Show current system settings (name, location, contact)
cargo run -p trendnet-teg3102ws-config -- system-settings

# Update system location
cargo run -p trendnet-teg3102ws-config -- system-settings --location "Lab Environment"

# Reboot the switch
cargo run -p trendnet-teg3102ws-config -- reboot

# Show the MAC address forwarding table
cargo run -p trendnet-teg3102ws-config -- mac-table

# Show detailed port configuration and status
cargo run -p trendnet-teg3102ws-config -- ports

# Show CLI (SSH/Telnet) configuration
cargo run -p trendnet-teg3102ws-config -- show-cli

# Enable or disable SSH/Telnet
cargo run -p trendnet-teg3102ws-config -- set-cli --ssh true --telnet false
```

**Spanning Tree Protocol (STP)**
```bash
# Show current STP status (including Root Bridge and counters)
cargo run -p trendnet-teg3102ws-config -- show-stp

# Show RSTP per-port roles and states (Active only in RSTP mode)
cargo run -p trendnet-teg3102ws-config -- rstp-ports

# Show CIST per-port roles and states (Active only in MSTP mode)
cargo run -p trendnet-teg3102ws-config -- cist-ports

# Enable STP with a specific protocol and priority
cargo run -p trendnet-teg3102ws-config -- set-stp --enable true --protocol rstp --priority 32768
```

**Port Configuration**
```bash
# Update multiple settings on a specific port
cargo run -p trendnet-teg3102ws-config -- set-port --id 1 --eap-passthrough true --description "Uplink Port" --pvid 1

# Disable a port
cargo run -p trendnet-teg3102ws-config -- set-port --id 2 --enable false

# Configure a port (or multiple ports) as an RSTP Edge Port and set a specific Path Cost
cargo run -p trendnet-teg3102ws-config -- set-rstp-port --id 1,2,3,5 --edge true --path-cost 1

# Show EEE configuration
cargo run -p trendnet-teg3102ws-config -- show-eee

# Set EEE configuration for a port
cargo run -p trendnet-teg3102ws-config -- set-eee --port 1 --enable true
```

**Loopback Detection (LBD)**
```bash
# Show current LBD status
cargo run -p trendnet-teg3102ws-config -- show-lbd

# Disable LBD
cargo run -p trendnet-teg3102ws-config -- set-lbd --enable false
```

**Security & Advanced**
```bash
# Show Jumbo Frame configuration
cargo run -p trendnet-teg3102ws-config -- show-jumbo-frame

# Set Jumbo Frame size (uses _extended API)
cargo run -p trendnet-teg3102ws-config -- set-jumbo-frame --size 9216

# Show DHCP Snooping configuration
cargo run -p trendnet-teg3102ws-config -- show-dhcp-snooping

# Enable DHCP Snooping
cargo run -p trendnet-teg3102ws-config -- set-dhcp-snooping --enable true

# Set a port as Trusted for DHCP Snooping
cargo run -p trendnet-teg3102ws-config -- set-dhcp-trust --id 1 --state trusted

# Show Storm Control configuration
cargo run -p trendnet-teg3102ws-config -- show-storm-control

# Set Storm Control configuration for a port
cargo run -p trendnet-teg3102ws-config -- set-storm-control --port 1 --broadcast 1024

# Show DoS protection configuration
cargo run -p trendnet-teg3102ws-config -- show-dos

# Enable DoS protection
cargo run -p trendnet-teg3102ws-config -- set-dos --enable true

# Show QoS configuration
cargo run -p trendnet-teg3102ws-config -- show-qos

# Enable QoS
cargo run -p trendnet-teg3102ws-config -- set-qos --enable true

# Show LLDP configuration
cargo run -p trendnet-teg3102ws-config -- show-lldp

# Enable LLDP
cargo run -p trendnet-teg3102ws-config -- set-lldp --enable true
```

**Multicast**
```bash
# Show IGMP snooping configuration
cargo run -p trendnet-teg3102ws-config -- show-igmp

# Enable IGMP snooping
cargo run -p trendnet-teg3102ws-config -- set-igmp --enable true

# Show global multicast filter configuration
cargo run -p trendnet-teg3102ws-config -- show-multicast-filter

# Enable Multicast Filter
cargo run -p trendnet-teg3102ws-config -- set-multicast-filter --enable true
```

**Persistence**
```bash
# Save changes to persistent flash memory
cargo run -p trendnet-teg3102ws-config -- save
```

## Running Tests

The project includes a suite of unit tests that use `wiremock` to simulate the switch's API.

```bash
cargo test -p trendnet-teg3102ws-config
```

## Library Usage

```rust
use trendnet_teg3102ws_config::{SwitchClient, StpConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SwitchClient::new("192.168.10.200")?;
    client.login("admin", "password").await?;
    
    let stp = client.get_stp_config().await?;
    println!("STP Enabled: {}", stp.enable);
    
    Ok(())
}
```
