# Network Debugging Project

The analysis and tools included here can guide an AI agent to configure a
TrendNet TEG3102-WS switch to serve as Ethernet backhaul for a wireless mesh
made of Google Nest WiFi Pro units.

## Problem Statement

This project aims to diagnose and resolve issues with wired backhaul connections for Google Nest WiFi Pro units.

For more details on the context and methodology, see:
* `prompt/goal.md` - The primary objective of this investigation.
* `prompt/approach.md` - The methodology for solving the problem.

## Network Topologies

Depending on the testing phase, the network operates in one of the following configurations:

* `prompt/live-setup.md` - The final home networking setup.
* `prompt/lab-setup.md` - The lab network designed for isolating and analyzing the switch.
* `prompt/direct-setup.md` - The lab network designed for directly analyzing the Nest WiFi Pro.

## Collected Information and Documentation

We have gathered extensive documentation and performed analyses on the network hardware:

### Hardware Analyses
* `docs/switch-basics.md` - Basic information about the managed switch.
* `docs/switch-analysis.md` - In-depth analysis of the switch's admin interface and API.
* `docs/modem-basics.md` - Basic information and architecture of the cable modem.
* `docs/modem-analysis.md` - In-depth analysis of the modem's admin interface.
* `docs/nest-wifi-analysis.md` - Analysis of Ethernet packet data emitted by the Nest WiFi Pro.

### Manuals
The original PDF manuals were converted into clean Markdown using [marker](https://github.com/datalab-to/marker).
* **Switch:** `docs/switch-manual/all.pdf` (Original) | `docs/switch-manual/all.md` (Markdown)
* **Modem:** `docs/modem-manual/all.pdf` (Original) | `docs/modem-manual/all.md` (Markdown)

## Utility Code (Rust Crates)

The project directory is a Cargo workspace hosting custom tools to interact with and monitor the network hardware:

* [`crates/nest-wifi-pro-ethernet`](crates/nest-wifi-pro-ethernet/README.md) - A tool for detecting and analyzing packets from the Nest WiFi Pro.
* [`crates/netgear-cm3000-config`](crates/netgear-cm3000-config/README.md) - A configuration utility for the Netgear cable modem.
* [`crates/trendnet-teg3102ws-config`](crates/trendnet-teg3102ws-config/README.md) - A configuration utility and API client for the TRENDnet switch.
