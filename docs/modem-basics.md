# Modem Basics: Netgear CM3000

## Product Information

*   **Model:** Netgear Nighthawk CM3000
*   **Hardware Version:** 1.01
*   **Firmware Version:** V6.01.04
*   **Cable Firmware Version:** V6.01.04
*   **Admin Console URL:** `http://192.168.100.1`

## Manuals

*   **User Manual (PDF):** [Download PDF](https://www.downloads.netgear.com/files/GDC/CM3000/CM3000_UM_EN.pdf)

## Admin Console Architecture

The admin console is a web-based interface that follows a "Static HTML + Dynamic JS Injection" pattern.

### Authentication Flow

1.  **Session Initiation:** A `GET` request to `/` provides an `XSRF_TOKEN` cookie and a dynamic `id` parameter embedded in the login form's `action` attribute (e.g., `/goform/Login?id=12345678`).
2.  **Login Submission:** A `POST` request to `/goform/Login?id=<dynamic_id>` with form data `loginName=admin&loginPassword=<password>` authenticates the session.
3.  **Session Management:** The `XSRF_TOKEN` cookie must be included in subsequent requests.

### Data Retrieval Pattern

The console generally does not use standard REST APIs or JSON. Instead, it relies on the following mechanisms:

*   **Pipe-Separated Strings:** Most status pages (e.g., `RouterStatus.htm`, `DashBoard.htm`) include a JavaScript function called `InitTagValue()`. This function returns a hardcoded string of data separated by pipes (`|`).
    ```javascript
    function InitTagValue() {
        var tagValueList = '1.01|V6.01.04|...';
        return tagValueList.split("|");
    }
    ```
*   **XML Fragments:** Some pages, such as the Event Log (`eventLog.htm`), embed XML-like strings within `InitTagValue()`.
*   **Dynamic Injection:** A frontend script (often `func.js` or `script.js`) parses these strings and updates the DOM elements by ID.

### Automation Strategy

To automate interactions or status collection with the modem:
1.  **Scraping:** Fetch the relevant `.htm` page and use regular expressions to extract the content of the `tagValueList` variable.
2.  **State Changes:** Identify the `goform` endpoint and parameters by inspecting the `<form>` elements and their `action` attributes.
3.  **Session Maintenance:** Always fetch the target page first to obtain the current dynamic ID and ensure the `XSRF_TOKEN` remains fresh.

## Backend Information

*   **Web Server:** `PS HTTP Server` (likely a proprietary embedded server).
*   **Backend Logic:** Handles requests via `/goform/` endpoints.
*   **OS Clues:** The server operates in a lightweight embedded environment. Frequent connection resets and occasional "Poor" status reports suggest the backend may be sensitive to high request volumes or network instability on the DOCSIS connection.
