# Nest WiFi Pro Ethernet Packet Analyzer

This tool detects and reports on specific packets emitted by Nest WiFi Pro units to support Ethernet backhaul discovery. It identifies devices using the Google OUI (`b8:7b:d4`) and extracts diagnostic metadata to help troubleshoot wired backhaul issues.

## Features

- **Detailed STP (Spanning Tree Protocol) Analysis**:
  - Detects 802.3 LLC frames with DSAP/SSAP `0x42`.
  - Identifies protocol versions (e.g., 802.1D, RSTP).
  - Displays **Root Bridge ID** and **Bridge ID** including their configured **priorities**.
  - Extracts the **Root Path Cost** (crucial for debugging Ethernet vs Wireless backhaul elections).
- **Specialized DHCP "Rouge" Probes**:
  - Detects DHCP Discovers containing the `gwifi_rouge_dhcp_detection` string.
  - Extracts and displays the **Transaction ID (XID)**.
- **mDNS Service Identification**:
  - Monitors multicast DNS traffic on port 5353.
  - Identifies specific services like **Matter**, **Google Cast**, **IPP**, and **Sleep Proxy**.
- **IPv6 Fabric Diagnostics**:
  - Detects ICMPv6 Router Advertisements (Type 134).
  - Extracts and displays the **Source IPv6 Link-Local Address**.
- **SSDP & ARP Monitoring**:
  - Detects SSDP discovery broadcasts and extracts the **Server** HTTP header.
  - Detects general ARP queries and extracts the **Sender IP** and **Target IP**.

## Usage

### Analyze a PCAP File
```bash
cargo run -p nest-wifi-pro-ethernet -- --pcap data/nest_backhaul.pcap
```

### Live Capture (Requires Sudo)
```bash
sudo ./target/debug/nest-check --interface enx9cbf0d005f94 --duration 65
```

You can optionally add the `--log-details` flag to print real-time debugging information about every MAC address (and its OUI vendor) observed on the network.

```bash
sudo ./target/debug/nest-check --interface enx9cbf0d005f94 --duration 65 --log-details
```

### Running without Sudo (Recommended for Local Dev)
On Linux, you can grant the binary specific capabilities (`CAP_NET_RAW` and `CAP_NET_ADMIN`) so you don't have to use `sudo` every time.

1. Build the tool:
   ```bash
   cargo build -p nest-wifi-pro-ethernet
   ```
2. Grant capabilities to the binary:
   ```bash
   sudo setcap cap_net_raw,cap_net_admin=eip ./target/debug/nest-check
   ```
3. Run normally:
   ```bash
   ./target/debug/nest-check --interface enx9cbf0d005f94
   ```

**Note:** You must re-run the `setcap` command every time you rebuild the binary, as the filesystem metadata is lost when the file is overwritten.
