# Netgear CM3000 Configuration Utility

A Rust-based CLI tool and library for programmatically interacting with the Netgear Nighthawk CM3000 Cable Modem's local web management interface.

## Features

- **Status & Monitoring**: Retrieve basic dashboard status, detailed DOCSIS channel statistics, and system event logs.
- **Management Operations**: Remote reboot and factory reset operations.
- **Advanced Configuration**: Enable or disable Link Aggregation (LACP) and HTTPS-only access.
- **Network Troubleshooting**: Set the starting downstream frequency and trigger log refreshes.
- **Security**: Update the local administrator password.

## Prerequisites

- **Rust**: Ensure you have the Rust toolchain installed.
- **Connectivity**: The modem must be reachable over the network (typically at `192.168.100.1` by default).

## Building

```bash
cargo build -p netgear-cm3000-config
```

## CLI Usage

The binary `netgear-cm3000-config` wraps the library's functionality. By default, it looks for credentials in `data/network.json`.

### Global Options
- `-c, --config <PATH>`: Path to the network configuration JSON file (default: `data/network.json`).

### Commands

**Monitoring & Status**
```bash
# View basic status from the modem's dashboard
cargo run -p netgear-cm3000-config -- status

# View detailed DOCSIS channel and signal status
cargo run -p netgear-cm3000-config -- docsis

# View the modem's event logs
cargo run -p netgear-cm3000-config -- logs
```

**Operations**
```bash
# Refresh the modem's event logs
cargo run -p netgear-cm3000-config -- refresh

# Reboot the cable modem
cargo run -p netgear-cm3000-config -- reboot

# Reset the modem to its factory default settings
cargo run -p netgear-cm3000-config -- factory-reset
```

**Configuration**
```bash
# Set the starting frequency (in Hz) for the downstream channel search
cargo run -p netgear-cm3000-config -- set-frequency 650000000

# Set a new administrator password for the modem
cargo run -p netgear-cm3000-config -- set-password "new_secure_password"

# Enable or disable Link Aggregation Control Protocol (LACP)
cargo run -p netgear-cm3000-config -- set-lacp true

# Enable or disable forced HTTPS for the local management interface
cargo run -p netgear-cm3000-config -- set-https true
```

## Running Tests

The project includes a suite of unit tests.

```bash
cargo test -p netgear-cm3000-config
```
