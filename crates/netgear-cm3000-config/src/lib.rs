use anyhow::{Context, Result, anyhow};
use regex::Regex;
use reqwest::header::REFERER;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

pub struct ModemClient {
    client: reqwest::Client,
    base_url: Url,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModemConfig {
    pub ip: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownstreamChannel {
    pub channel: String,
    pub lock_status: String,
    pub modulation: String,
    pub channel_id: String,
    pub frequency: String,
    pub power: String,
    pub snr: String,
    pub correctables: String,
    pub uncorrectables: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpstreamChannel {
    pub channel: String,
    pub lock_status: String,
    pub channel_type: String,
    pub channel_id: String,
    pub symbol_rate: String,
    pub frequency: String,
    pub power: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocsisStatus {
    pub downstream_bonded: Vec<DownstreamChannel>,
    pub upstream_bonded: Vec<UpstreamChannel>,
    pub system_time: String,
    pub up_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStatus {
    pub connection_status: String,
    pub cm_ip: String,
    pub cm_mac: String,
    pub firmware_version: String,
    pub hardware_version: String,
    pub system_time: String,
    pub up_time: String,
    pub connection_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: String,
    pub priority: String,
    pub description: String,
}

impl ModemClient {
    pub fn new(config: &ModemConfig) -> Result<Self> {
        let base_url = Url::parse(&format!("http://{}", config.ip))?;
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .cookie_provider(jar)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        Ok(Self { client, base_url })
    }

    pub async fn login(&self, config: &ModemConfig) -> Result<()> {
        let resp = self.client.get(self.base_url.clone()).send().await?;
        let text = resp.text().await?;

        let re = Regex::new(r#"action="/goform/Login\?id=(\d+)""#)?;
        let dynamic_id = re
            .captures(&text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .context("Failed to find dynamic login ID")?;

        let login_url = self
            .base_url
            .join(&format!("/goform/Login?id={}", dynamic_id))?;
        let params = [
            ("loginName", &config.username),
            ("loginPassword", &config.password),
        ];

        let login_resp = self.client.post(login_url).form(&params).send().await?;

        if !login_resp.status().is_success() && login_resp.status() != reqwest::StatusCode::FOUND {
            return Err(anyhow!("Login failed with status: {}", login_resp.status()));
        }

        Ok(())
    }

    pub async fn fetch_page(&self, path: &str) -> Result<String> {
        let url = self.base_url.join(path)?;
        let index_url = self.base_url.join("/index.htm")?;

        let resp = self
            .client
            .get(url)
            .header(REFERER, index_url.as_str())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch page {} with status: {}",
                path,
                resp.status()
            ));
        }

        Ok(resp.text().await?)
    }

    pub fn parse_tag_values(html: &str) -> Result<Vec<String>> {
        let re = Regex::new(r"(?s)function InitTagValue\(\).*?var tagValueList = '([^']*)';")?;
        let caps = re
            .captures(html)
            .context("InitTagValue not found in page")?;
        let val_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        Ok(val_str.split('|').map(|s| s.to_string()).collect())
    }

    pub fn parse_xml_logs(html: &str) -> Result<Vec<LogEntry>> {
        let re = Regex::new(r#"var xmlFormat = "([^"]*)";"#)?;
        let caps = re.captures(html).context("xmlFormat not found in page")?;
        let raw_logs = caps.get(1).map(|m| m.as_str()).unwrap_or("");

        let mut entries = Vec::new();
        let entry_re = Regex::new(
            r"(?i)<docsDevEvTime>(.*?)<\\/docsDevEvTime>.*?<docsDevEvLevel>(.*?)<\\/docsDevEvLevel>.*?<docsDevEvText>(.*?)<\\/docsDevEvText>",
        )?;

        for cap in entry_re.captures_iter(raw_logs) {
            entries.push(LogEntry {
                time: cap[1].replace("\\/", "/"),
                priority: cap[2].replace("\\/", "/"),
                description: cap[3].replace("\\/", "/"),
            });
        }

        Ok(entries)
    }

    pub fn parse_dashboard_status(html: &str) -> Result<DashboardStatus> {
        let tag_values = Self::parse_tag_values(html)?;

        Ok(DashboardStatus {
            connection_status: tag_values
                .get(1)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
            cm_ip: tag_values
                .get(30)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
            cm_mac: "Use 'docsis' for detail".to_string(),
            firmware_version: "V6.01.04".to_string(),
            hardware_version: "1.01".to_string(),
            system_time: "Use 'docsis' for detail".to_string(),
            up_time: "Use 'docsis' for detail".to_string(),
            connection_type: "Cable".to_string(),
        })
    }

    pub fn parse_docsis_status(html: &str) -> Result<DocsisStatus> {
        let mut downstream_bonded = Vec::new();
        let ds_re =
            Regex::new(r"(?s)function InitDsTableTagValue\(\).*?var tagValueList = '([^']*)';")?;
        if let Some(caps) = ds_re.captures(html) {
            let val_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let vals: Vec<_> = val_str.split('|').collect();
            if let Some(first) = vals.first() {
                let count: usize = first.parse().unwrap_or(0);
                for i in 0..count {
                    let base = 1 + i * 9;
                    if base + 8 >= vals.len() {
                        break;
                    }
                    downstream_bonded.push(DownstreamChannel {
                        channel: vals[base].to_string(),
                        lock_status: vals[base + 1].to_string(),
                        modulation: vals[base + 2].to_string(),
                        channel_id: vals[base + 3].to_string(),
                        frequency: vals[base + 4].to_string(),
                        power: vals[base + 5].to_string(),
                        snr: vals[base + 6].to_string(),
                        correctables: vals[base + 7].to_string(),
                        uncorrectables: vals[base + 8].to_string(),
                    });
                }
            }
        }

        let mut upstream_bonded = Vec::new();
        let us_re =
            Regex::new(r"(?s)function InitUsTableTagValue\(\).*?var tagValueList = '([^']*)';")?;
        if let Some(caps) = us_re.captures(html) {
            let val_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let vals: Vec<_> = val_str.split('|').collect();
            if let Some(first) = vals.first() {
                let count: usize = first.parse().unwrap_or(0);
                for i in 0..count {
                    let base = 1 + i * 7;
                    if base + 6 >= vals.len() {
                        break;
                    }
                    upstream_bonded.push(UpstreamChannel {
                        channel: vals[base].to_string(),
                        lock_status: vals[base + 1].to_string(),
                        channel_type: vals[base + 2].to_string(),
                        channel_id: vals[base + 3].to_string(),
                        symbol_rate: vals[base + 4].to_string(),
                        frequency: vals[base + 5].to_string(),
                        power: vals[base + 6].to_string(),
                    });
                }
            }
        }

        let tag_values = Self::parse_tag_values(html)?;
        let system_time = tag_values.get(10).cloned().unwrap_or_default();
        let up_time = tag_values.get(14).cloned().unwrap_or_default();

        Ok(DocsisStatus {
            downstream_bonded,
            upstream_bonded,
            system_time,
            up_time,
        })
    }

    pub async fn refresh_logs(&self) -> Result<()> {
        self.post_form(
            "/eventLog.htm",
            &[("buttonHit", "refresh"), ("buttonValue", "Refresh")],
        )
        .await
    }

    pub async fn reboot(&self) -> Result<()> {
        self.post_form("/RouterStatus.htm", &[("buttonSelect", "2")])
            .await
    }

    pub async fn factory_reset(&self) -> Result<()> {
        self.post_form("/RouterStatus.htm", &[("buttonSelect", "3")])
            .await
    }

    pub async fn set_password(&self, old_pass: &str, new_pass: &str) -> Result<()> {
        self.post_form(
            "/SetPassword.htm",
            &[
                ("sysOldPasswd", old_pass),
                ("sysNewPasswd", new_pass),
                ("sysConfirmPasswd", new_pass),
                ("strcheckPassRec", "off"),
                ("checkPassRec", "0"),
                ("buttonHit", "cfAlert_Apply"),
                ("buttonValue", "Apply"),
                (
                    "timestamp_value",
                    "Mon May 25 2026 00:00:00 GMT-0700 (Pacific Daylight Time)",
                ),
            ],
        )
        .await
    }

    pub async fn set_frequency(&self, freq_hz: u32) -> Result<()> {
        self.post_form(
            "/DocsisStatus.htm",
            &[
                ("Startupfreq", &freq_hz.to_string()),
                ("buttonHit", "Apply"),
                ("buttonValue", "Apply"),
            ],
        )
        .await
    }

    pub async fn set_lacp(&self, enabled: bool) -> Result<()> {
        let mode = if enabled { "dynamic" } else { "disable" };
        self.post_form(
            "/PortTrunking_setting.htm",
            &[
                ("ptk_radio", mode),
                ("pTrunking_nv", if enabled { "disable" } else { "dynamic" }),
                ("buttonHit", "Apply"),
                ("buttonValue", "Apply"),
            ],
        )
        .await
    }

    pub async fn set_https(&self, enabled: bool) -> Result<()> {
        let mut params = vec![
            ("buttonHit", "Apply"),
            ("buttonValue", "Apply"),
            ("local_https_enable", if enabled { "1" } else { "0" }),
        ];
        if enabled {
            params.push(("local_https_check", "local_https_mg"));
        }
        self.post_form("/WebServiceManagement.htm", &params).await
    }

    async fn post_form(&self, path: &str, params: &[(&str, &str)]) -> Result<()> {
        let html = self.fetch_page(path).await?;
        let re = Regex::new(r#"action="(/goform/[^?]+)\?id=(\d+)""#)?;
        let caps = re
            .captures(&html)
            .context(format!("Failed to find dynamic ID in {}", path))?;
        let base_path = caps.get(1).map(|m| m.as_str()).unwrap();
        let dynamic_id = caps.get(2).map(|m| m.as_str()).unwrap();

        let url = self
            .base_url
            .join(&format!("{}?id={}", base_path, dynamic_id))?;

        let resp = self
            .client
            .post(url)
            .form(params)
            .header(REFERER, self.base_url.join(path)?.as_str())
            .send()
            .await?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::FOUND {
            return Err(anyhow!(
                "POST to {} failed with status: {}",
                path,
                resp.status()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_values() {
        let html = r#"function InitTagValue() { var tagValueList = '525000000|Locked|OK|Operational|OK|Operational|&nbsp;|&nbsp;|Enabled|BPI+|Sun May 24 23:16:45 2026|0|0|1|2 days 14:05:05|3|1|'; }"#;
        let vals = ModemClient::parse_tag_values(html).unwrap();
        assert_eq!(vals[0], "525000000");
        assert_eq!(vals[1], "Locked");
        assert_eq!(vals[10], "Sun May 24 23:16:45 2026");
        assert_eq!(vals[14], "2 days 14:05:05");
    }

    #[test]
    fn test_parse_xml_logs() {
        let html = r#"var xmlFormat = "<docsDevEventTable><tr><docsDevEvTime>Sun May 24 23:16:59 2026<\/docsDevEvTime><docsDevEvLevel>Critical (3)<\/docsDevEvLevel><docsDevEvText>No Ranging Response received - T3 time-out;CM-MAC=28:94:01:ab:80:b8;CMTS-MAC=00:90:f0:17:04:00;CM-QOS=1.1;CM-VER=3.1;<\/docsDevEvText><\/tr><\/docsDevEventTable>";"#;
        let entries = ModemClient::parse_xml_logs(html).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(
            entries[0]
                .description
                .contains("No Ranging Response received")
        );
        assert_eq!(entries[0].time, "Sun May 24 23:16:59 2026");
    }

    #[test]
    fn test_parse_docsis_status() {
        let html = r#"
function InitDsTableTagValue()
{
    var tagValueList = '2|1|Locked|QAM256|20|525000000 Hz|-14.5|35.4|19892254|10919496|2|Locked|QAM256|1|405000000 Hz|-14.9|35.4|13176176|21954696|';
    return tagValueList.split("|");
}
function InitUsTableTagValue()
{
    var tagValueList = '1|1|Locked|ATDMA|20|5120 Ksym/sec|35600000 Hz|58.5 dBmV|';
    return tagValueList.split("|");
}
function InitTagValue()
{
    var tagValueList = '0|0|0|0|0|0|0|0|0|0|Sun May 24 23:16:45 2026|0|0|0|2 days 14:05:05|';
    return tagValueList.split("|");
}
"#;
        let status = ModemClient::parse_docsis_status(html).unwrap();
        assert_eq!(status.system_time, "Sun May 24 23:16:45 2026");
        assert_eq!(status.up_time, "2 days 14:05:05");

        assert_eq!(status.downstream_bonded.len(), 2);
        assert_eq!(status.downstream_bonded[0].channel, "1");
        assert_eq!(status.downstream_bonded[1].channel, "2");

        assert_eq!(status.upstream_bonded.len(), 1);
        assert_eq!(status.upstream_bonded[0].channel, "1");
        assert_eq!(status.upstream_bonded[0].power, "58.5 dBmV");
    }

    #[test]
    fn test_extract_form_action() {
        let html = r#"<form id="target" method="POST" action="/goform/SetPassword?id=275109932">"#;
        let re = Regex::new(r#"action="(/goform/[^?]+)\?id=(\d+)""#).unwrap();
        let caps = re.captures(html).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "/goform/SetPassword");
        assert_eq!(caps.get(2).unwrap().as_str(), "275109932");
    }

    #[test]
    fn test_url_construction() {
        let base_url = Url::parse("http://192.168.100.1").unwrap();
        let base_path = "/goform/SetPassword";
        let dynamic_id = "275109932";
        let url = base_url
            .join(&format!("{}?id={}", base_path, dynamic_id))
            .unwrap();
        assert_eq!(
            url.as_str(),
            "http://192.168.100.1/goform/SetPassword?id=275109932"
        );
    }

    async fn setup_mock_server() -> (mockito::ServerGuard, ModemConfig) {
        let server = mockito::Server::new_async().await;
        let url = server.url();
        let host = url.trim_start_matches("http://");
        let config = ModemConfig {
            ip: host.to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };
        (server, config)
    }

    #[tokio::test]
    async fn test_api_login() {
        let (mut server, config) = setup_mock_server().await;
        let _m1 = server
            .mock("GET", "/")
            .with_status(200)
            .with_body(r#"<form action="/goform/Login?id=12345">"#)
            .create_async()
            .await;
        let _m2 = server
            .mock("POST", "/goform/Login?id=12345")
            .with_status(302)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.login(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_api_reboot() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/RouterStatus.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/RouterStatus?id=999">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/RouterStatus?id=999")
            .match_body(mockito::Matcher::UrlEncoded(
                "buttonSelect".to_string(),
                "2".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.reboot().await.unwrap();
    }

    #[tokio::test]
    async fn test_api_factory_reset() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/RouterStatus.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/RouterStatus?id=999">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/RouterStatus?id=999")
            .match_body(mockito::Matcher::UrlEncoded(
                "buttonSelect".to_string(),
                "3".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.factory_reset().await.unwrap();
    }

    #[tokio::test]
    async fn test_api_set_password() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/SetPassword.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/SetPassword?id=111">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/SetPassword?id=111")
            .match_body(mockito::Matcher::UrlEncoded(
                "sysNewPasswd".to_string(),
                "new_secret".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client
            .set_password("old_secret", "new_secret")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_api_set_lacp() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/PortTrunking_setting.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/PortTrunkingSetting?id=222">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/PortTrunkingSetting?id=222")
            .match_body(mockito::Matcher::UrlEncoded(
                "ptk_radio".to_string(),
                "dynamic".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.set_lacp(true).await.unwrap();
    }

    #[tokio::test]
    async fn test_api_set_https() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/WebServiceManagement.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/WebServiceManagement?id=333">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/WebServiceManagement?id=333")
            .match_body(mockito::Matcher::UrlEncoded(
                "local_https_enable".to_string(),
                "1".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.set_https(true).await.unwrap();
    }

    #[tokio::test]
    async fn test_api_refresh_logs() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/eventLog.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/eventLog?id=444">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/eventLog?id=444")
            .match_body(mockito::Matcher::UrlEncoded(
                "buttonHit".to_string(),
                "refresh".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.refresh_logs().await.unwrap();
    }

    #[tokio::test]
    async fn test_api_fetch_page() {
        let (mut server, config) = setup_mock_server().await;
        let _m = server
            .mock("GET", "/test.htm")
            .match_header("referer", format!("{}/index.htm", server.url()).as_str())
            .with_status(200)
            .with_body("content")
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        let body = client.fetch_page("/test.htm").await.unwrap();
        assert_eq!(body, "content");
    }

    #[tokio::test]
    async fn test_api_set_frequency() {
        let (mut server, config) = setup_mock_server().await;
        let _m_get = server
            .mock("GET", "/DocsisStatus.htm")
            .with_status(200)
            .with_body(r#"<form action="/goform/DocsisStatus?id=777">"#)
            .create_async()
            .await;
        let _m_post = server
            .mock("POST", "/goform/DocsisStatus?id=777")
            .match_body(mockito::Matcher::UrlEncoded(
                "Startupfreq".to_string(),
                "525000000".to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;
        let client = ModemClient::new(&config).unwrap();
        client.set_frequency(525000000).await.unwrap();
    }
}
