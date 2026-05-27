# Modem Analysis: Netgear CM3000

## HTML Pages Inventory

### Core Pages
- **Login.htm**: The initial landing page for authentication.
- **index.htm**: The main frame/container for the authenticated session (often redirects if accessed directly).
- **DashBoard.htm**: High-level status overview of the modem (Internet, Voice, Wireless, etc.).
- **DocsisStatus.htm**: Detailed DOCSIS signal levels, channel lock status, and startup procedure.
- **eventLog.htm**: System events and diagnostic messages from the cable modem.
- **PortTrunking_setting.htm**: Configuration for Ethernet Port Aggregation (LACP).
- **downloadDebugLog.htm**: Initiates the export of diagnostic log files.
- **DocsisOffline.htm**: Shown when the modem is searching for a signal or offline.
- **RouterStatus.htm**: Advanced gateway status summary; contains controls for Reboot and Factory Reset.
- **SetPassword.htm**: Page for changing the admin password.
- **WebServiceManagement.htm**: Local management settings, including "Force to use HTTPS".
- **NETGEAR_CM3000-Console.log**: Binary/encrypted debug log file.

## Endpoints & Form Inventory

### Mandatory Parameters
For any automated interaction, the following must be provided:
- **`id` (Query Parameter)**: A dynamic session ID found in the `<form>`'s `action` attribute. It varies per page load.
- **`XSRF_TOKEN` (Cookie)**: A security token set as a cookie (e.g., `XSRF_TOKEN=30030007`). It must be included in the request headers.

### POST Endpoints (/goform/)

#### **Login**
- **Source Page**: `Login.htm` (or `/`)
- **URL**: `/goform/Login?id=<DynamicID>`
- **Variables**:
    - `loginName`: The admin username. Known value: `admin`.
    - `loginPassword`: The admin password.
- **Purpose**: Establishes an authenticated session.

#### **RouterStatus**
- **Source Page**: `RouterStatus.htm` (or the ADVANCED tab)
- **URL**: `/goform/RouterStatus?id=<DynamicID>`
- **Variables**:
    - `buttonSelect`:
        - `2`: Reboot.
        - `3`: Factory Reset.
- **Purpose**: System administration tasks (Reboot/Reset).

#### **CableConnection**
- **Source Page**: `DocsisStatus.htm`
- **URL**: `/goform/DocsisStatus?id=<DynamicID>`
- **Variables**:
    - `Startupfreq`: The starting frequency for downstream channel scanning (in Hz).
    - `buttonHit`: `Apply`.
    - `buttonValue`: `Apply`.
- **Purpose**: Manually specifying the initial frequency for DOCSIS synchronization.

#### **SetPassword**
- **Source Page**: `SetPassword.htm`
- **URL**: `/goform/SetPassword?id=<DynamicID>`
- **Variables**:
    - `sysOldPasswd`: Current admin password.
    - `sysNewPasswd`: New admin password.
    - `sysConfirmPasswd`: Confirmation of the new password.
    - `strcheckPassRec`: `on` or `off` (Enable/Disable password recovery).
    - `checkPassRec`: `1` or `0` (Enable/Disable password recovery).
    - `question1`, `answer1`, `question2`, `answer2`: Recovery questions and answers.
    - `timestamp_value`: A timestamp string (e.g., JS `Date().toString()`).
    - `buttonHit`: `cfAlert_Apply`.
    - `buttonValue`: `Apply`.
- **Purpose**: Updates the `admin` account password and configures recovery.

#### **eventLog**
- **Source Page**: `eventLog.htm`
- **URL**: `/goform/eventLog?id=<DynamicID>`
- **Variables**:
    - `buttonHit`: The name of the button triggered. Known values:
        - `docsDevEvControl.0`: Clears the log.
        - `refresh`: Refreshes the log view.
    - `buttonValue`: The internal action code. Known values:
        - `1`: Triggered by "Clear Log".
        - `Refresh`: Triggered by "Refresh".
- **Purpose**: Manages the system event log state.

#### **PortTrunkingSetting**
- **Source Page**: `PortTrunking_setting.htm`
- **URL**: `/goform/PortTrunkingSetting?id=<DynamicID>`
- **Variables**:
    - `ptk_radio`: The LACP mode. Known values:
        - `disable`: Port aggregation off.
        - `dynamic`: LACP (IEEE 802.3ad) on.
    - `pTrunking_nv`: The *previous* state (must be sent to toggle).
    - `buttonHit`: `Apply`.
    - `buttonValue`: `Apply`.
- **Purpose**: Configures Ethernet Port Aggregation (LACP).

#### **WebServiceManagement**
- **Source Page**: `WebServiceManagement.htm`
- **URL**: `/goform/WebServiceManagement?id=<DynamicID>`
- **Variables**:
    - `local_https_enable`: `1` (Enable) or `0` (Disable).
    - `local_https_check`: `local_https_mg` (only if enabling).
    - `buttonHit`: `Apply`.
    - `buttonValue`: `Apply`.
- **Purpose**: Manages local web service settings, specifically forcing HTTPS.

## Guidance for AI Automation

### 1. Handling Authentication & Session
The modem uses a stateful session identified by a cookie and a dynamic form ID.
- **Initial Step**: Perform a `GET /` to obtain the `XSRF_TOKEN` cookie and the dynamic `id` from the login form's `action` attribute.
- **Authentication**: `POST /goform/Login?id=<ID>` with `loginName=admin` and `loginPassword=<PASSWORD>`.
- **Persistence**: Use a cookie jar for all subsequent requests.
- **Referer Requirement**: Most pages require a `Referer` header pointing to the page containing the form or `index.htm`.

### 2. Data Extraction Pattern
The modem does not use REST/JSON. Instead, it employs **Dynamic JavaScript Injection** and **Server-Side HTML Generation**:

#### **JavaScript Injection (InitTagValue)**
Most status data is stored in a `tagValueList` variable inside a script block.
- **Pattern**: `function InitTagValue\(\).*?var tagValueList = '([^']*)';`
- **Dashboard (`DashBoard.htm`) Indices**:
    - `1`: Connection Status (e.g., "Good", "Poor").
    - `30`: Cable Modem IP Address.
- **DOCSIS (`DocsisStatus.htm`) Indices**:
    - `10`: Current System Time.
    - `14`: System Up Time.

#### **JavaScript Injection (Bonded Channels)**
Detailed channel tables are populated via specialized functions:
- `InitDsTableTagValue()`: Downstream Bonded Channels.
- `InitUsTableTagValue()`: Upstream Bonded Channels.
- `InitDsOfdmTableTagValue()`: Downstream OFDM Channels.
- `InitUsOfdmaTableTagValue()`: Upstream OFDMA Channels.

These functions contain a `tagValueList` where the first element is the number of rows, followed by N rows of fixed-length data.
- **DS Row Length**: 9 fields (Channel, Lock Status, Modulation, ID, Freq, Power, SNR, Correctables, Uncorrectables).
- **US Row Length**: 7 fields (Channel, Lock Status, Type, ID, Symbol Rate, Freq, Power).

#### **Event Logs (xmlFormat)**
Logs are embedded as an XML-like string in the `xmlFormat` variable in `eventLog.htm`.
- **Format**: `<tr><docsDevEvTime>...<\/docsDevEvTime><docsDevEvLevel>...<\/docsDevEvLevel><docsDevEvText>...<\/docsDevEvText><\/tr>`
- **Parsing**: Use regex to extract the time, priority, and text from each `<tr>` block.

#### **HTML Scraping fallback**
Some fields (like MAC Address on `RouterStatus.htm`) are best retrieved by scraping the HTML elements directly using IDs like `#InternetMAC`, `#InternetIP`, etc.

## Management Avenues

### Rust Management Tool (`netgear-cm3000-config`)
A Rust-based library and CLI tool have been implemented in `crates/netgear-cm3000-config`.
- **Functionality**:
    - Automatic session management.
    - Status data extraction (Dashboard overview, detailed DOCSIS).
    - Event log retrieval and tabular parsing.
    - Administrative tasks: Reboot, Factory Reset, Set Password, Set Frequency, Set LACP, Set HTTPS.
- **Usage**:
    ```bash
    cargo run -p netgear-cm3000-config -- status            # High-level health
    cargo run -p netgear-cm3000-config -- logs              # Formatted event logs
    cargo run -p netgear-cm3000-config -- docsis            # Detailed signal analysis
    cargo run -p netgear-cm3000-config -- reboot            # Reboot modem
    cargo run -p netgear-cm3000-config -- set-lacp true     # Enable port aggregation
    ```

## Diagnostic Findings (Project Specific)

Current analysis of the live modem reveals:
1.  **Poor Signal Levels**:
    - Downstream Power: ~-15 dBmV (Optimal: 0 +/- 7).
    - Upstream Power: ~58.5 dBmV (Optimal: < 50).
2.  **Frequent T3 Timeouts**: The logs are saturated with "No Ranging Response received - T3 time-out", which is directly caused by the high upstream transmit power required to reach the CMTS through line noise or attenuation.
3.  **Partial Service**: The modem often enters "Partial Service" mode on the upstream due to its inability to maintain locks on all channels at these power levels.

## Reverse Engineering Guide for AI Agents

To uncover additional features or deeper diagnostic data from this modem, future AI agents should follow these specialized strategies.

### 1. Static Analysis of HTML and JavaScript
The modem's logic is heavily reliant on server-side injection into static HTML templates.

- **Identify POST Endpoints**: Grep the source code for `goform`. Every administrative action is a POST to `/goform/<Action>`.
- **Session ID Pattern**: Note the `?id=` query parameter in form actions. This is dynamic and must be scraped from the current page before every POST.
- **Data Mapping**: Look for functions like `InitTagValue()` or `InitUpdateView()`. These functions map pipe-separated indices in a `tagValueList` string to specific UI elements (e.g., `tagValues[10]` -> `CurrentSystemTime`).
- **Table Construction**: Technical tables (like bonded channels) are often built in JS using a leading "count" field followed by N rows of fixed-length data. Search for `InitDsTableTagValue` for the primary example.
- **Hidden Inputs**: Always check the `<form>` blocks for `<input type="hidden">`. The modem frequently uses these to track internal state or "previous" values (e.g., `pTrunking_nv` in port aggregation).

### 2. Dynamic Exploration with Chrome DevTools
Using a live browser context is the most effective way to see the modem's "true" state.

- **Form Discovery**: Use `evaluate_script` to execute `document.forms` and dump all inputs, actions, and methods. This bypasses any obfuscation or complex JS-based form construction.
- **Global Variable Inspection**: Check the state of the `window` object. Variables like `tagValueList` or `xmlFormat` are often populated once per page load and represent the raw backend data before JS formatting.
- **Interception and Observation**:
    - Use `click` on an "Apply" button, then immediately use `list_network_requests` to see the outgoing POST body and headers.
    - Check for `302 Found` redirects, which often indicate a successful administrative action even if the page content doesn't immediately change.
- **State Change Verification**: After a configuration POST, navigate back to the summary page and use `evaluate_script` to check if the relevant field in `tagValueList` has updated.
- **UI Scraping fallback**: If the JS mapping is too complex, use `take_snapshot` to get the accessibility tree. Identifying an element by its text label and then finding its UID is a reliable way to map backend values to human-readable names.

## Resource and Power Management

An exhaustive review of the Netgear CM3000 manual and its administrative interface revealed no software-configurable power-saving features. Unlike advanced switches that support Energy Efficient Ethernet (EEE), the cable modem operates at a constant baseline power to maintain its DOCSIS carrier locks.

- **Overheating Mitigation**: The manufacturer explicitly notes that if the Power LED lights solid red, the modem is overheating. The only supported resolution is to physically disconnect the power adapter, allow the unit to cool in a well-ventilated area, and reconnect it.
