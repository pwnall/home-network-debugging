# Router/Switch Analysis: TRENDnet TEG-3102WS

## System Overview
- **Model**: TRENDnet TEG-3102WS (10-Port Multi-Gig Web Smart Switch)
- **Web UI**: React-based frontend (Single Page Application).
- **API**: REST-like API exposed at `/api/`.
- **CLI**: Standard networking CLI accessible via SSH (port 22) or Console.

## Interaction Methods

### 1. Web UI & REST API (Recommended for Automation)

The switch's web interface communicates exclusively via a JSON API. This is the most reliable way to programmatically manage the switch.

#### Authentication
Authentication is session-based using a JSON Web Token (JWT).
- **Login Endpoint**: `PATCH /api/system/login`
- **Payload**: `{"user": "admin", "password": "your_password"}`
- **Response**:
  ```json
  {
    "restful_res": {
      "token": "eyJhbGci...", 
      "utctimestamp": 946688727,
      "timeout": 300,
      "errCode": 0,
      "message": "OK"
    }
  }
  ```
- **Header**: All subsequent requests must include the header: `Authorization: Bearer <token>`.
- **Note**: The login method is `PATCH`, not `POST`.

#### Common Endpoints
| Feature | Method | Endpoint | Description |
| :--- | :--- | :--- | :--- |
| System Status | GET | `/api/system/status` | Hardware/Firmware info. |
| System Settings | GET | `/api/system/settings` | Device name, contact, etc. |
| Port Config | GET/PATCH | `/api/ports` | Detailed port settings (VLAN, speed, EAP). |
| STP Global | GET/PATCH | `/api/stp` | Enable/Disable STP and global timers. |
| RSTP Info | GET | `/api/stp/rstp` | Port roles/states in **RSTP mode**. (Err 2715 in MSTP) |
| CIST Info | GET | `/api/stp/cist` | Port roles/states in **MSTP mode**. (Err 6004 in RSTP) |
| MAC Table | GET | `/api/macaddr` | Current forwarding database. |
| LBD | GET/PATCH | `/api/lbd` | Loopback Detection settings. |
| CLI Session | GET/PATCH | `/api/system/settings/session/cli` | Enable/Disable SSH and Telnet. |
| IGMP Snooping | GET/PATCH | `/api/igmp` | IGMP Snooping settings. |
| Multicast Filter| GET/PATCH | `/api/multicastfilter` | Global Multicast Filtering. |
| DHCP Snooping | GET | `/api/system/settings/dhcpsnp` | Global DHCP Snooping settings. |
| DHCP Trust | GET | `/api/system/settings/dhcpsnp/trustports` | Per-port DHCP trust status. |
| Jumbo Frames | GET | `/api/system/settings/jumboframe` | MTU settings. |
| Storm Control | GET | `/api/ports/stormcontrol` | Per-port traffic throttling. |
| DoS Protect | GET | `/api/system/settings/dos` | Denial of Service protection. |
| LLDP | GET | `/api/lldp` | Link Layer Discovery Protocol. |

### Feature Relevance Analysis

The following features were evaluated for their impact on Nest WiFi Pro wired backhaul:

1. **Spanning Tree Protocol (STP)**: **High Relevance**. Essential for loop prevention and wired backhaul detection. Must be enabled with high priority (61440) to allow Nest units to be Root.
2. **DHCP Snooping**: **High Relevance**. Could block critical Nest DHCP traffic if misconfigured. Currently disabled globally.
3. **Storm Control**: **Medium Relevance**. Could throttle Nest's noisy discovery traffic. Currently disabled (0) on all ports.
4. **DoS Protection**: **Medium Relevance**. Could misinterpret Nest heartbeats as attacks. Currently disabled.
5. **IGMP Snooping / Multicast Filtering**: **Medium Relevance**. Nest uses multicast for discovery. Currently disabled, which is safe as it defaults to flooding behavior.
6. **Jumbo Frames**: **Low Relevance**. Currently 1522 bytes, sufficient for standard backhaul.
7. **EEE (802.3az)**: **Low Relevance**. Can cause link stability issues. Currently disabled on all ports.
8. **LLDP**: **Low Relevance**. Currently disabled.

#### Data Pattern
- Most configuration changes use `PATCH`.
- **API Wrapper Inconsistency**: While some early analysis suggested a generic `body` wrapper, modern firmware primarily uses:
    - **Direct Arrays**: Many endpoints (like `/api/ports`, `/api/stp/rstp`, `/api/ports/stormcontrol`) expect a top-level JSON array of configuration objects (e.g., `[{"portID": "1", ...}]`).
    - **Flat Objects**: System-level settings often expect a flat object (e.g., `{"deviceName": "..."}`).
    - **Avoid the `body` key**: Including a `{"body": ...}` wrapper often results in the switch returning success (errCode 0) but silently ignoring the update.
- **Extended Endpoints**: The web UI often uses `_extended` versions of endpoints for updates (e.g., `/api/system/settings/jumboframe_extended`, `/api/ports/eee_extended`). These usually accept the simplified payload formats.
- Always check the `errCode` in the response (0 is success).

### 2. Command Line Interface (CLI)

The CLI provides a familiar Cisco-like experience.

#### Access
- **SSH**: Port 22. Must be enabled via the Web UI/API first (see `sshEnable` in `/api/system/settings/session/cli`).
- **Interactive Login**: Requires username/password.
- **Non-interactive Execution**: Use `sshpass` for automated commands, but note that the CLI is highly interactive and may require PTY allocation (`ssh -tt`).

#### Command Structure
- `?`: Lists available commands.
- `show <module>`: Displays information.
- `configure`: Enters configuration mode.
- `interface <id>`: Scopes configuration to a specific port.
- `write`: Saves running configuration to flash.

### 3. Debugging Backhaul Issues (Nest WiFi Pro)

The Nest WiFi Pro relies on the **Spanning Tree Protocol (STP)** to detect wired loops and confirm a valid backhaul path.

#### Key Findings
- **STP State**: If STP is disabled on the switch, the switch may drop 802.1D BPDUs instead of flooding them.
- **BPDU Inspection**: Use `tcpdump -e -i <interface> stp` to see the source MAC addresses.
  - If the source MAC belongs to the switch (`78:2d:7e...`), the switch is participating in STP.
  - If the source MAC belongs to Google (`b8:7b:d4...`), the switch is transparently flooding BPDUs (or the Nest is the Root).
- **Bridge Priority**: To allow the Nest WiFi Pro to remain the Root Bridge (Priority 30976), the switch priority should be set to a higher value (lower precedence), such as **61440**.
- **Recommendation**: To ensure Nest backhaul works, **Enable STP** (preferably RSTP) on the switch, set the priority to `61440`, and configure all ports connected to Nest units as **Edge Ports**.

#### STP Port Blocking Issue
In a mesh environment, Nest WiFi Pro units maintain a wireless backhaul while simultaneously attempting to establish a wired backhaul. This creates a physical loop that standard Spanning Tree algorithms will detect and block.
- **Symptom**: Ports connected to secondary Nest units show as `Alternate/Discarding` in the `rstp-ports` output. This occurs because the secondary Nest units advertise a better Root Path Cost (via their wireless connection to the root bridge) than the default wired path cost from the switch (e.g., 20000 for 1G). As a result, the switch yields and blocks its ports.
- **Solution**: 
  1. **Critical Step**: Artificially lower the wired Path Cost by setting the **Path Cost** to `1` on **ALL switch ports connected to any Nest WiFi unit** (both primary and secondary units). Because any Nest unit might win the STP Root Bridge election (e.g., due to lower priority or MAC address), setting all of them to `1` ensures that no matter which unit is Root, the switch always advertises a near-zero Root Path Cost to the other units. This guarantees the switch becomes the Designated or Root bridge for all links, forcing secondary Nest units to prefer the wired backhaul and block their wireless links.
  2. Configure all ports on the switch connected to Nest units as **RSTP Edge Ports** (`adminEdge=true`). This ensures the switch ports rapidly transition to `Forwarding` without waiting for standard STP timers.
- **Verification**: Run `cargo run -p trendnet-teg3102ws-config -- rstp-ports`. Ensure none of the Nest unit ports are in an `Alternate/Discarding` state. Depending on which unit won the Root election, one port should be the `Root` port (Forwarding), and the others should be `Designated` (Forwarding).

## Automation Tool: `trendnet-teg3102ws-config`

A Rust-based utility is available in the `crates/` directory to automate switch management.

### Features
- JWT session management and automatic token inclusion.
- Configuration of STP, Loopback Detection, and SSH/Telnet access.
- Non-interactive CLI for CI/CD or scripted network adjustments.

### Example Commands
```bash
# Get current system status and MAC table
cargo run -p trendnet-teg3102ws-config -- status
cargo run -p trendnet-teg3102ws-config -- mac-table

# Enable STP with RSTP protocol and lowest priority (61440)
cargo run -p trendnet-teg3102ws-config -- set-stp --enable true --protocol rstp --priority 61440

# Check per-port roles (ensure port to Nest is 'Root')
cargo run -p trendnet-teg3102ws-config -- rstp-ports

# Save changes to persistent flash
cargo run -p trendnet-teg3102ws-config -- save
```

## Lab Verification (May 2026)

The configuration recommended above was applied to a new TRENDnet TEG-3102WS switch to verify reproducibility.

### Configuration Applied
- **STP Protocol**: RSTP (Rapid Spanning Tree)
- **Bridge Priority**: 61440
- **LBD**: Disabled
- **IGMP Snooping**: Disabled (default)

### Observed Results
1. **Root Bridge Identification**: Upon enabling RSTP, the switch correctly identified the Nest WiFi Pro Router (`b8:7b:d4:a8:43:e7`) as the Root Bridge.
2. **Packet Propagation**: 
   - **STP BPDUs**: The switch originated its own BPDUs (from MAC `78:2d:7e:27:03:e9`) identifying the Nest Router as the Root.
   - **DHCP Rouge Probes**: Successfully observed `gwifi_rouge_dhcp_detection` packets being flooded to the admin host.
   - **mDNS/IPv6**: Multicast traffic from Nest units was correctly delivered to the monitoring port without requiring explicit IGMP join.
   - **SSDP and ARP**: When the Nest WiFi Pro was acting as a router connected to the cable modem, it was observed emitting SSDP (UDP port 1900) discovery packets and ARP requests to query connected devices. These were also successfully delivered through the switch.

### Key Learning
While `eapPassthrough` was investigated to see if it affected the detection of additional units, it did not change the results of the `nest-check` tool in this environment. The most critical setting for backhaul stability remains the **STP Priority** to ensure the switch does not contest the Root Bridge role.

**Capture Duration**: When running diagnostic tools like `nest-check` or manual captures (e.g., `tcpdump`) to debug switch traffic, ensure the capture duration is at least 65 seconds. This guarantees capturing all periodic broadcast and multicast packets, particularly SSDP probes which are emitted exactly every 60 seconds.

## Reverse Engineering the Switch API

### Method 1: Live Observation with Chrome DevTools

When automating new features or troubleshooting existing ones, use Chrome DevTools to observe the actual interaction between the web frontend and the switch backend.

#### 1. Discovery Methodology
To discover the API for a specific feature:
1. **Navigate**: Use `navigate_page` to go to the relevant GUI section (e.g., `http://<ip>/gui/network/lbd`).
2. **Observe Requests**: Call `list_network_requests` to see the traffic. Look for `GET` requests to `/api/...` that occur during page load.
3. **Trigger Action**: Perform the configuration change in the UI (e.g., toggle a switch and click "Apply") and call `list_network_requests` again.
4. **Inspect Payload**: Use `get_network_request` on the `PATCH` or `POST` request. **Crucially, observe if the data is wrapped in a key or sent as a raw object/array.**

#### 2. Dynamic API Exploration
You can use `evaluate_script` to test API hypotheses directly from the browser context, which already has the correct origin and authentication context (if you've logged in):
```javascript
async () => {
  const token = "..."; // Get from a previous request header or page state
  const resp = await fetch('/api/ports', {
    method: 'PATCH',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify([{ "portID": "1", "description": "DevTools Test" }])
  });
  return await resp.json();
}
```

### Method 2: Static Analysis of JavaScript Bundles

If an action isn't clear from network traffic alone, or to discover hidden endpoints, analyze the frontend's minified logic.

#### 1. Identify and Fetch the Bundle
The switch uses a React-based Single Page Application (SPA). Core logic resides in minified JavaScript files (e.g., `/static/js/main.3a004a2c.chunk.js`). Use `evaluate_script` or `curl` to read these files.

#### 2. Search for URL Patterns and Constants
Search for strings containing `/api/` or common keywords (e.g., `dhcp`, `stp`, `jumbo`). You will find a mapping object containing methods and URLs:
```javascript
systemJumboframePatch:{method:"patch",url:"/system/settings/jumboframe_extended"}
```

#### 3. Identify Logical Actions
Look for Redux/State action constants. The switch follows a consistent pattern:
- `LOAD_<FEATURE>` for data retrieval.
- `UPDATE_<FEATURE>` or `CREATE_<FEATURE>` for configuration changes.

#### 4. Determine Payload Construction
Search for the key (e.g., `systemJumboframePatch`) to see how the code prepares the data before the API call. This is the most reliable way to know if an endpoint expects:
- **Flat Objects**: e.g., `{"enable": true}`.
- **Keyed Wrappers**: e.g., `{"igmpConfs": { ... }}`.
- **Direct Arrays**: e.g., `[{"portID": "1", ...}]`.
- **Legacy Body Wrappers**: Some older endpoints might still use `{"body": { ... }}`.

#### 5. Analyze the Communication Helper (`va`)
The UI uses a centralized helper function (often named `va`) to handle all API calls. Finding its definition reveals how it interprets the mapping object, which headers it adds, and how it handles errors.

### Common Pitfalls
- **Token Expiry**: The JWT token expires. If you receive `401 Unauthorized`, re-run the login sequence.
- **Silent Failures**: The switch often returns `errCode: 0` even if the payload structure is slightly wrong. Always verify the change with a subsequent `GET` request.
- **Rebooting**: The `/api/system/reboot` endpoint is a `POST` request. It may require a specific JSON body. Testing this will disconnect the DevTools session.
- **Hardware vs Software Status**: Some fields in `GET` responses are read-only status (e.g., `linkSpeed`) while others are configuration (`linkSpeedConf`). Ensure your `PATCH` payloads only include configurable fields.

## Resource and Power Management

To minimize the odds of overheating-induced failures and reduce CPU utilization, you can leverage the switch's built-in power-saving features and disable unnecessary per-packet processing.

### Energy Efficient Ethernet (EEE)
The switch supports Energy Efficient Ethernet (IEEE 802.3az). When enabled on a per-port basis, the switch will dynamically scale down power consumption during periods of low network traffic.
- **Support**: Available on all standard copper ports (1-8).
- **Configuration Tool**: Use `cargo run -p trendnet-teg3102ws-config -- set-eee --port <ID> --enable true` to manage this setting programmatically.
- **Recommendation**: Ensure EEE is enabled globally to maintain a cooler operating temperature for the networking equipment.

### Per-Packet Processing Features
The switch offers several features that require deep inspection or filtering of individual packets (snooping, QoS, Storm Control, DoS protection, etc.). If these features are not actively required for the network topology, they should be disabled to conserve CPU resources and lower the operating temperature.

The following features have been verified as **Disabled** in this environment to prioritize switch stability and power conservation:
- **IGMP Snooping** (`show-igmp`)
- **Multicast Filtering** (`show-multicast-filter`)
- **DHCP Snooping** (`show-dhcp-snooping`)
- **Storm Control** (`show-storm-control`)
- **DoS Protection** (`show-dos`)
- **Quality of Service (QoS)** (`show-qos`)
- **Loopback Detection (LBD)** (`show-lbd`)
- **Link Layer Discovery Protocol (LLDP)** (`show-lldp`)

### Physical Port Management
While explicitly disabling disconnected ports (`cargo run -p trendnet-teg3102ws-config -- set-port --id <ID> --enable false`) completely eliminates idle PHY power draw, it breaks plug-and-play functionality because the port will not detect when a cable is inserted.
- **Recommendation**: To maintain convenience without sacrificing too much efficiency, leave all unused physical ports **Enabled** and rely on the globally configured **Energy Efficient Ethernet (EEE)** feature to automatically place idle ports into a low-power standby state.
- **Note**: Logical trunk/aggregation ports (t1-t8) cannot be disabled via the port API and will return API errors if attempted.

### Thermal Mitigation and PoE Scheduling
- **Passive Cooling**: The TEG-3102WS is a fanless model. Internal CPU load and power draw directly dictate the chassis temperature.
- **Physical Clearance**: The switch requires at least 10cm (4 inches) of physical clearance at the front and rear for passive ventilation.
- **PoE Scheduling**: If Power over Ethernet (PoE) devices are connected, consider using the Web UI to configure **PoE Time Range** schedules. Automatically disabling PoE power to endpoints (like IP cameras or access points) outside of business hours is a highly effective way to lower baseline energy consumption and heat generation.
