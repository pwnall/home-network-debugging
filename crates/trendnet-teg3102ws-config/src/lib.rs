use anyhow::{Context, Result, anyhow};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub switch: SwitchConfig,
}

#[derive(Debug, Deserialize)]
pub struct SwitchConfig {
    pub ip: String,
    pub username: String,
    pub password: String,
}

pub struct SwitchClient {
    client: reqwest::Client,
    base_url: Url,
    token: Option<String>,
}

#[derive(Serialize)]
struct LoginPayload<'a> {
    user: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct RestfulRes<T> {
    restful_res: T,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    #[serde(rename = "errCode")]
    err_code: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalStpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(rename = "forwardDelay", skip_serializing_if = "Option::is_none")]
    pub forward_delay: Option<u32>,
    #[serde(rename = "maximumAge", skip_serializing_if = "Option::is_none")]
    pub maximum_age: Option<u32>,
    #[serde(rename = "txHoldCount", skip_serializing_if = "Option::is_none")]
    pub tx_hold_count: Option<u32>,
    #[serde(rename = "helloTime", skip_serializing_if = "Option::is_none")]
    pub hello_time: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StpPortCounter {
    #[serde(rename = "portID")]
    pub port_id: String,
    #[serde(rename = "rxBPDU")]
    pub rx_bpdu: u64,
    #[serde(rename = "txBPDU")]
    pub tx_bpdu: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootBridge {
    pub cost: u32,
    #[serde(rename = "portNo")]
    pub port_no: String,
    pub priority: u32,
    #[serde(rename = "rootAddr")]
    pub root_addr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StpConfig {
    pub enable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(rename = "rootBridge", skip_serializing_if = "Option::is_none")]
    pub root_bridge: Option<RootBridge>,
    #[serde(rename = "globalStpSettings", skip_serializing_if = "Option::is_none")]
    pub global_stp_settings: Option<GlobalStpSettings>,
    #[serde(rename = "stpPortCounter", skip_serializing_if = "Option::is_none")]
    pub stp_port_counter: Option<Vec<StpPortCounter>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RstpPortConfig {
    #[serde(rename = "portID")]
    pub port_id: String,
    pub priority: u32,
    #[serde(rename = "pathCost")]
    pub path_cost: u32,
    #[serde(rename = "pathCostOper", skip_serializing_if = "Option::is_none")]
    pub path_cost_oper: Option<u32>,
    #[serde(rename = "edgePortConf")]
    pub edge_port_conf: String,
    #[serde(rename = "edgePortOper", skip_serializing_if = "Option::is_none")]
    pub edge_port_oper: Option<String>,
    #[serde(rename = "portRole", skip_serializing_if = "Option::is_none")]
    pub port_role: Option<String>,
    #[serde(rename = "portState", skip_serializing_if = "Option::is_none")]
    pub port_state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CistPortConfig {
    #[serde(rename = "portID")]
    pub port_id: String,
    pub priority: u32,
    #[serde(rename = "pathCost")]
    pub path_cost: u32,
    #[serde(rename = "pathCostOper", skip_serializing_if = "Option::is_none")]
    pub path_cost_oper: Option<u32>,
    #[serde(rename = "portRole", skip_serializing_if = "Option::is_none")]
    pub port_role: Option<String>,
    #[serde(rename = "portState", skip_serializing_if = "Option::is_none")]
    pub port_state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LbdConfig {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliConfig {
    #[serde(rename = "sshEnable")]
    pub ssh_enable: bool,
    #[serde(rename = "telnetEnable")]
    pub telnet_enable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSettings {
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "systemDescription")]
    pub system_description: String,
    #[serde(rename = "systemLocation")]
    pub system_location: String,
    #[serde(rename = "systemContact")]
    pub system_contact: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MgmtInterface {
    #[serde(rename = "IP")]
    pub ip: String,
    pub configuration: String,
    #[serde(rename = "defaultGateway")]
    pub default_gateway: String,
    #[serde(rename = "dhcpOption43")]
    pub dhcp_option_43: String,
    #[serde(rename = "dns1IP")]
    pub dns1_ip: String,
    #[serde(rename = "dns2IP")]
    pub dns2_ip: String,
    pub submask: String,
    #[serde(rename = "uplinkPort")]
    pub uplink_port: String,
    #[serde(rename = "vlanID")]
    pub vlan_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStatus {
    #[serde(rename = "deviceName")]
    pub device_name: String,
    #[serde(rename = "firmwareVersion")]
    pub firmware_version: String,
    #[serde(rename = "macAddr")]
    pub mac_addr: String,
    #[serde(rename = "serialNo")]
    pub serial_no: String,
    #[serde(rename = "modelName")]
    pub model_name: String,
    pub temperature: i32,
    #[serde(rename = "upTime")]
    pub up_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MacEntry {
    #[serde(rename = "vlanID")]
    pub vlan_id: u32,
    #[serde(rename = "macAddr")]
    pub mac_addr: String,
    #[serde(rename = "portID")]
    pub port_id: String,
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortConfig {
    #[serde(rename = "portID")]
    pub port_id: String,
    pub enable: bool,
    pub description: String,
    #[serde(rename = "linkSpeedConf")]
    pub link_speed_conf: String,
    #[serde(rename = "linkSpeed", skip_serializing_if = "Option::is_none")]
    pub link_speed: Option<String>,
    #[serde(rename = "flowControl")]
    pub flow_control: bool,
    pub isolation: bool,
    pub pvid: u32,
    #[serde(rename = "802_1p")]
    pub dot1p: u32,
    pub tagged_vlans: String,
    pub untagged_vlans: String,
    #[serde(rename = "acceptFrameType")]
    pub accept_frame_type: String,
    #[serde(rename = "ingressFilter")]
    pub ingress_filter: bool,
    pub mtu: u32,
    #[serde(rename = "portSecMaxCount")]
    pub port_sec_max_count: u32,
    #[serde(rename = "eapPassthrough")]
    pub eap_passthrough: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgmpConfig {
    pub enable: bool,
    #[serde(rename = "reportSuppression")]
    pub report_suppression: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MulticastFilterConfig {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DhcpSnoopingVlan {
    pub enable: bool,
    #[serde(rename = "vlanID")]
    pub vlan_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DhcpSnoopingConfig {
    pub enable: bool,
    #[serde(rename = "macVerify")]
    pub mac_verify: bool,
    pub vlan: Vec<DhcpSnoopingVlan>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DhcpSnoopingTrustPort {
    #[serde(rename = "portID")]
    pub port_id: String,
    #[serde(rename = "State")]
    pub state: String, // "trusted" or "untrusted"
    #[serde(rename = "Action")]
    pub action: String, // "drop" or "forward"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JumboFrameConfig {
    #[serde(rename = "frameSize")]
    pub frame_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StormControlEntry {
    #[serde(rename = "portNo")]
    pub port_no: u32,
    pub broadcast: u64,
    #[serde(rename = "unknownMulticast")]
    pub unknown_multicast: u64,
    #[serde(rename = "unknownUnicast")]
    pub unknown_unicast: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DosConfig {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QosConfig {
    pub enable: bool,
    #[serde(rename = "scheduleMethod", skip_serializing_if = "Option::is_none")]
    pub schedule_method: Option<String>,
    #[serde(rename = "trustMode", skip_serializing_if = "Option::is_none")]
    pub trust_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LldpGlobalConfig {
    pub enable: bool,
    pub holdtime: u32,
    #[serde(rename = "reinitDelay")]
    pub reinit_delay: u32,
    #[serde(rename = "transmitDelay")]
    pub transmit_delay: u32,
    #[serde(rename = "transmitInterval")]
    pub transmit_interval: u32,
    pub version: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EeeEntry {
    pub enable: bool,
    #[serde(rename = "portNo")]
    pub port_no: u32,
}

#[derive(Deserialize)]
struct GenericResponse {
    #[serde(rename = "errCode")]
    err_code: i32,
    #[serde(default)]
    message: String,
}

impl RestfulRes<GenericResponse> {
    fn check(&self) -> Result<()> {
        if self.restful_res.err_code != 0 {
            return Err(anyhow!(
                "API error {}: {}",
                self.restful_res.err_code,
                self.restful_res.message
            ));
        }
        Ok(())
    }
}

impl SwitchClient {
    pub fn new(ip: &str) -> Result<Self> {
        let base_url = Url::parse(&format!("http://{}/api/", ip))?;
        Ok(Self {
            client: reqwest::Client::new(),
            base_url,
            token: None,
        })
    }

    pub async fn login(&mut self, user: &str, password: &str) -> Result<()> {
        let url = self.base_url.join("system/login")?;
        let payload = LoginPayload { user, password };

        let resp: RestfulRes<LoginResponse> = self
            .client
            .patch(url)
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        if resp.restful_res.err_code != 0 {
            return Err(anyhow!(
                "Login failed with error code: {}",
                resp.restful_res.err_code
            ));
        }

        self.token = Some(resp.restful_res.token);
        Ok(())
    }

    fn auth_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))?,
            );
        } else {
            return Err(anyhow!("Not authenticated. Call login() first."));
        }
        Ok(headers)
    }

    async fn get_request<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.base_url.join(path)?;
        let resp: serde_json::Value = self
            .client
            .get(url)
            .headers(self.auth_headers()?)
            .send()
            .await?
            .json()
            .await?;

        if let Some(err_code) = resp
            .get("restful_res")
            .and_then(|r| r.get("errCode"))
            .and_then(|e| e.as_i64())
            && err_code != 0
        {
            return Err(anyhow!("API error {} on path {}", err_code, path));
        }

        let restful_res = resp.get("restful_res").context("Missing restful_res")?;
        Ok(serde_json::from_value(restful_res.clone())?)
    }

    pub async fn change_password(
        &self,
        username: &str,
        new_password: &str,
        is_first_login: bool,
        old_password: Option<&str>,
    ) -> Result<()> {
        let url = self.base_url.join("system/settings/account")?;

        let payload = if is_first_login {
            serde_json::json!({
                "isFirstChangePwd": true,
                "accountConfs": [{
                    "userName": username,
                    "password": new_password,
                    "privilegeType": "Admin"
                }]
            })
        } else {
            let old_pwd = old_password.unwrap_or("");
            serde_json::json!([{
                "userName": username,
                "password": new_password,
                "oldPassword": old_pwd,
                "privilegeType": "Admin"
            }])
        };

        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.token.as_ref().unwrap()),
            )
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        if resp.restful_res.err_code != 0 {
            return Err(anyhow::anyhow!("API Error: {}", resp.restful_res.message));
        }
        Ok(())
    }

    pub async fn get_mgmt_interface(&self) -> Result<MgmtInterface> {
        let res: serde_json::Value = self.get_request("system/settings/mgmtinterface").await?;
        let mgmt = res.get("mgmtInterface").context("Missing mgmtInterface")?;
        Ok(serde_json::from_value(mgmt.clone())?)
    }

    pub async fn patch_mgmt_interface(&self, config: &MgmtInterface) -> Result<()> {
        let url = self.base_url.join("system/settings/mgmtinterface")?;
        let req = self
            .client
            .patch(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.token.as_ref().unwrap()),
            )
            .json(&config);

        let new_ip = config.ip.clone();

        let patch_task = async {
            match req.send().await {
                Ok(resp) => {
                    if let Ok(parsed) = resp.json::<RestfulRes<GenericResponse>>().await
                        && parsed.restful_res.err_code != 0 {
                            return Err(anyhow::anyhow!("API Error: {}", parsed.restful_res.message));
                        }
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Request error: {}", e)),
            }
        };

        let poll_task = async {
            let new_url = format!("http://{}/", new_ip);
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(1))
                .build()
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            loop {
                if let Ok(res) = client.get(&new_url).send().await
                    && res.status().is_success() {
                        return Ok::<(), anyhow::Error>(());
                    }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        };

        let (patch_res, poll_res) = tokio::join!(
            async {
                tokio::time::timeout(std::time::Duration::from_secs(10), patch_task)
                    .await
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("Patch task timed out")))
            },
            async {
                tokio::time::timeout(std::time::Duration::from_secs(15), poll_task)
                    .await
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("Poll task timed out")))
            }
        );

        if poll_res.is_ok() {
            Ok(())
        } else if let Err(e) = patch_res {
            if e.to_string().contains("API Error") {
                Err(e)
            } else {
                Err(anyhow::anyhow!(
                    "Failed to verify IP change. Patch error: {}",
                    e
                ))
            }
        } else {
            poll_res
        }
    }

    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        let res: serde_json::Value = self.get_request("system/status").await?;
        let status = res.get("systemInfo").context("Missing systemInfo")?;
        Ok(serde_json::from_value(status.clone())?)
    }

    pub async fn get_system_settings(&self) -> Result<SystemSettings> {
        let res: serde_json::Value = self.get_request("system/settings").await?;
        let settings = res.get("systemConfs").context("Missing systemConfs")?;
        Ok(serde_json::from_value(settings.clone())?)
    }

    pub async fn patch_system_settings(&self, settings: &SystemSettings) -> Result<()> {
        let url = self.base_url.join("system/settings")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(settings)
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn reboot(&self) -> Result<()> {
        let url = self.base_url.join("system/reboot")?;
        let _resp = self
            .client
            .post(url)
            .headers(self.auth_headers()?)
            .json(&json!({ "reboot": true })) // Trial and error body
            .send()
            .await?;

        // We don't check the response because the switch will reboot and close the connection
        Ok(())
    }

    pub async fn get_mac_table(&self) -> Result<Vec<MacEntry>> {
        let res: serde_json::Value = self.get_request("macaddr").await?;
        let table = res
            .get("macaddrConfs")
            .and_then(|c| c.get("macAddrTable"))
            .context("Missing macAddrTable")?;
        Ok(serde_json::from_value(table.clone())?)
    }

    pub async fn get_ports(&self) -> Result<Vec<PortConfig>> {
        let res: serde_json::Value = self.get_request("ports").await?;
        let ports = res.get("portConfs").context("Missing portConfs")?;
        Ok(serde_json::from_value(ports.clone())?)
    }

    pub async fn get_stp_config(&self) -> Result<StpConfig> {
        let res: serde_json::Value = self.get_request("stp").await?;
        let stp = res.get("stpConfs").context("Missing stpConfs")?;
        Ok(serde_json::from_value(stp.clone())?)
    }

    pub async fn get_rstp_ports(&self) -> Result<Vec<RstpPortConfig>> {
        let res: serde_json::Value = self.get_request("stp/rstp").await?;
        let ports = res
            .get("rstpPortConfs")
            .context("Missing rstpPortConfs (is STP in RSTP mode?)")?;
        Ok(serde_json::from_value(ports.clone())?)
    }

    pub async fn get_cist_ports(&self) -> Result<Vec<CistPortConfig>> {
        let res: serde_json::Value = self.get_request("stp/cist").await?;
        let ports = res
            .get("cistPortConfs")
            .context("Missing cistPortConfs (is STP in MSTP mode?)")?;
        Ok(serde_json::from_value(ports.clone())?)
    }

    pub async fn get_lbd_config(&self) -> Result<LbdConfig> {
        let res: serde_json::Value = self.get_request("lbd").await?;
        let lbd = res.get("lbdConfs").context("Missing lbdConfs")?;
        Ok(serde_json::from_value(lbd.clone())?)
    }

    pub async fn get_cli_config(&self) -> Result<CliConfig> {
        let res: serde_json::Value = self.get_request("system/settings/session/cli").await?;
        let cli = res.get("webConfs").context("Missing webConfs")?;
        Ok(serde_json::from_value(cli.clone())?)
    }

    pub async fn get_igmp_config(&self) -> Result<IgmpConfig> {
        let res: serde_json::Value = self.get_request("igmp").await?;
        let igmp = res.get("igmpConfs").context("Missing igmpConfs")?;
        Ok(serde_json::from_value(igmp.clone())?)
    }

    pub async fn get_multicast_filter_config(&self) -> Result<MulticastFilterConfig> {
        let res: serde_json::Value = self.get_request("multicastfilter").await?;
        let filter = res
            .get("mcastFilterConfs")
            .context("Missing mcastFilterConfs")?;
        Ok(serde_json::from_value(filter.clone())?)
    }

    pub async fn get_dhcp_snooping_config(&self) -> Result<DhcpSnoopingConfig> {
        let res: serde_json::Value = self.get_request("system/settings/dhcpsnp").await?;
        let snp = res.get("dhcpsnpConfs").context("Missing dhcpsnpConfs")?;
        Ok(serde_json::from_value(snp.clone())?)
    }

    pub async fn get_dhcp_snooping_trust_ports(&self) -> Result<Vec<DhcpSnoopingTrustPort>> {
        let res: serde_json::Value = self
            .get_request("system/settings/dhcpsnp/trustports")
            .await?;
        let ports = res.get("portStatus").context("Missing portStatus")?;
        Ok(serde_json::from_value(ports.clone())?)
    }

    pub async fn get_jumbo_frame_config(&self) -> Result<JumboFrameConfig> {
        let res: serde_json::Value = self.get_request("system/settings/jumboframe").await?;
        let jumbo = res
            .get("jumboFrameConfs")
            .context("Missing jumboFrameConfs")?;
        Ok(serde_json::from_value(jumbo.clone())?)
    }

    pub async fn get_storm_control_config(&self) -> Result<Vec<StormControlEntry>> {
        let res: serde_json::Value = self.get_request("ports/stormcontrol").await?;
        let storm = res
            .get("stormCtrlConfs")
            .context("Missing stormCtrlConfs")?;
        Ok(serde_json::from_value(storm.clone())?)
    }

    pub async fn get_dos_config(&self) -> Result<DosConfig> {
        let res: serde_json::Value = self.get_request("system/settings/dos").await?;
        let dos = res.get("dosConfs").context("Missing dosConfs")?;
        Ok(serde_json::from_value(dos.clone())?)
    }

    pub async fn get_qos_config(&self) -> Result<QosConfig> {
        let res: serde_json::Value = self.get_request("system/settings/qos").await?;
        let qos = res.get("qosConfs").context("Missing qosConfs")?;
        Ok(serde_json::from_value(qos.clone())?)
    }

    pub async fn get_lldp_config(&self) -> Result<LldpGlobalConfig> {
        let res: serde_json::Value = self.get_request("lldp").await?;
        let lldp = res
            .get("lldpGlobalConfs")
            .context("Missing lldpGlobalConfs")?;
        Ok(serde_json::from_value(lldp.clone())?)
    }

    pub async fn get_eee_config(&self) -> Result<Vec<EeeEntry>> {
        let res: serde_json::Value = self.get_request("ports/eee").await?;
        let eee = res.get("portEeeConfs").context("Missing portEeeConfs")?;
        Ok(serde_json::from_value(eee.clone())?)
    }

    pub async fn patch_eee_config(&self, port_no: u32, enable: bool) -> Result<()> {
        let url = self.base_url.join("ports/eee_extended")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&json!([
                    {
                        "portNo": port_no,
                        "enable": enable
                    }
            ]))
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_igmp_config(&self, config: &IgmpConfig) -> Result<()> {
        let url = self.base_url.join("igmp")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(config)
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_multicast_filter_config(
        &self,
        config: &MulticastFilterConfig,
    ) -> Result<()> {
        let url = self.base_url.join("multicastfilter")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(config)
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_dhcp_snooping_config(&self, config: &DhcpSnoopingConfig) -> Result<()> {
        let url = self.base_url.join("system/settings/dhcpsnp")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(config)
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_dhcp_snooping_trust_ports(&self, port_id: &str, state: &str) -> Result<()> {
        let url = self.base_url.join("system/settings/dhcpsnp/trustports")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&json!({
                "portID": port_id,
                "State": state
            }))
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_jumbo_frame_config(&self, frame_size: u32) -> Result<()> {
        let url = self.base_url.join("system/settings/jumboframe_extended")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&json!({
                "frameSize": frame_size
            }))
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_storm_control_config(
        &self,
        port_no: u32,
        broadcast: u64,
        unknown_multicast: u64,
        unknown_unicast: u64,
    ) -> Result<()> {
        let url = self.base_url.join("ports/stormcontrol")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&json!([
                {
                    "portNo": port_no,
                    "broadcast": broadcast,
                    "unknownMulticast": unknown_multicast,
                    "unknownUnicast": unknown_unicast
                }
            ]))
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_dos_config(&self, enable: bool) -> Result<()> {
        let url = self.base_url.join("system/settings/dos")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&json!({
                "enable": enable
            }))
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_qos_config(&self, config: &QosConfig) -> Result<()> {
        let url = self.base_url.join("system/settings/qos")?;
        let mut payload = json!({
            "enable": config.enable
        });
        if let Some(ref sm) = config.schedule_method {
            payload["scheduleMethod"] = json!(sm);
        }
        if let Some(ref tm) = config.trust_mode {
            payload["trustMode"] = json!(tm);
        }

        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_lldp_config(&self, enable: bool) -> Result<()> {
        let url = self.base_url.join("lldp")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&json!({
                "enable": enable
            }))
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn patch_stp_config(&self, config: &StpConfig) -> Result<()> {
        let url = self.base_url.join("stp")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(config)
            .send()
            .await?
            .json()
            .await?;

        resp.check()?;
        Ok(())
    }

    pub async fn patch_lbd_config(&self, config: &LbdConfig) -> Result<()> {
        let url = self.base_url.join("lbd")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(config)
            .send()
            .await?
            .json()
            .await?;

        resp.check()?;
        Ok(())
    }

    pub async fn patch_cli_config(&self, config: &CliConfig) -> Result<()> {
        let url = self.base_url.join("system/settings/session/cli")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(config)
            .send()
            .await?
            .json()
            .await?;

        resp.check()?;
        Ok(())
    }

    pub async fn patch_rstp_config(
        &self,
        port_id: &str,
        edge_port: Option<bool>,
        path_cost: Option<u32>,
    ) -> Result<()> {
        let url = self.base_url.join("stp/rstp")?;
        let mut port_obj = serde_json::Map::new();
        port_obj.insert("portID".to_string(), json!(port_id));
        if let Some(ep) = edge_port {
            port_obj.insert(
                "edgePortConf".to_string(),
                json!(if ep { "Yes" } else { "No" }),
            );
        }
        if let Some(pc) = path_cost {
            port_obj.insert("pathCost".to_string(), json!(pc));
        }
        let payload = json!([port_obj]);

        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        resp.check()?;
        Ok(())
    }

    pub async fn patch_port_config(&self, config: &PortConfig) -> Result<()> {
        let url = self.base_url.join("ports")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .patch(url)
            .headers(self.auth_headers()?)
            .json(&vec![config])
            .send()
            .await?
            .json()
            .await?;
        resp.check()?;
        Ok(())
    }

    pub async fn save_config(&self) -> Result<()> {
        let url = self.base_url.join("system/save")?;
        let resp: RestfulRes<GenericResponse> = self
            .client
            .post(url)
            .headers(self.auth_headers()?)
            .send()
            .await?
            .json()
            .await?;

        if resp.restful_res.err_code != 0 {
            return Err(anyhow!(
                "Failed to save config: {}",
                resp.restful_res.message
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_login() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/system/login"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "token": "test-token",
                    "errCode": 0
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.login("admin", "password123").await?;
        assert_eq!(client.token, Some("test-token".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_system_status() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/system/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "systemInfo": {
                        "deviceName": "TEG-3102WS",
                        "firmwareVersion": "v1.00.16",
                        "macAddr": "78:2d:7e:24:3b:08",
                        "serialNo": "EP5A3A1000052",
                        "modelName": "TEG-3102WS",
                        "temperature": 0,
                        "upTime": 5505
                    },
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        let status = client.get_system_status().await?;
        assert_eq!(status.device_name, "TEG-3102WS");
        assert_eq!(status.firmware_version, "v1.00.16");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_mac_table() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/macaddr"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "macaddrConfs": {
                        "macAddrTable": [
                            {
                                "vlanID": 1,
                                "macAddr": "00:ae:f7:5d:9c:72",
                                "portID": "1",
                                "mode": "Dynamic"
                            }
                        ]
                    },
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        let table = client.get_mac_table().await?;
        assert_eq!(table.len(), 1);
        assert_eq!(table[0].mac_addr, "00:ae:f7:5d:9c:72");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_ports() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ports"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "portConfs": [
                        {
                            "portID": "1",
                            "enable": true,
                            "description": "Test Port",
                            "linkSpeedConf": "Auto",
                            "linkSpeed": "1G",
                            "flowControl": true,
                            "isolation": false,
                            "pvid": 1,
                            "802_1p": 0,
                            "tagged_vlans": "",
                            "untagged_vlans": "1",
                            "acceptFrameType": "acceptAll",
                            "ingressFilter": false,
                            "mtu": 1522,
                            "portSecMaxCount": 0,
                            "eapPassthrough": true
                        }
                    ],
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        let ports = client.get_ports().await?;
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port_id, "1");
        assert_eq!(ports[0].mtu, 1522);
        assert_eq!(ports[0].link_speed_conf, "Auto");
        assert_eq!(ports[0].link_speed.as_deref(), Some("1G"));
        assert!(ports[0].eap_passthrough);
        Ok(())
    }

    #[tokio::test]
    async fn test_patch_port_config() -> Result<()> {
        let mock_server = MockServer::start().await;

        let config = PortConfig {
            port_id: "1".to_string(),
            enable: true,
            description: "New Description".to_string(),
            link_speed_conf: "Auto".to_string(),
            link_speed: None,
            flow_control: true,
            isolation: false,
            pvid: 1,
            dot1p: 0,
            tagged_vlans: "".to_string(),
            untagged_vlans: "1".to_string(),
            accept_frame_type: "acceptAll".to_string(),
            ingress_filter: false,
            mtu: 1522,
            port_sec_max_count: 0,
            eap_passthrough: true,
        };

        Mock::given(method("PATCH"))
            .and(path("/api/ports"))
            .and(wiremock::matchers::body_json(json!([config])))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        client.patch_port_config(&config).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_get_system_settings() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/system/settings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "systemConfs": {
                        "deviceName": "TEG-3102WS",
                        "systemDescription": "TRENDnet TEG-3102WS",
                        "systemLocation": "Lab",
                        "systemContact": "Admin"
                    },
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        let settings = client.get_system_settings().await?;
        assert_eq!(settings.device_name, "TEG-3102WS");
        assert_eq!(settings.system_location, "Lab");
        Ok(())
    }

    #[tokio::test]
    async fn test_patch_jumbo_frame_config() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/system/settings/jumboframe_extended"))
            .and(wiremock::matchers::body_json(json!({"frameSize": 9216})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        client.patch_jumbo_frame_config(9216).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_reboot() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/system/reboot"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        client.reboot().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_patch_rstp_config() -> Result<()> {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/stp/rstp"))
            .and(wiremock::matchers::body_json(json!([{
                "portID": "1",
                "edgePortConf": "Yes",
                "pathCost": 1
            }])))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "restful_res": {
                    "errCode": 0,
                    "message": "OK"
                }
            })))
            .mount(&mock_server)
            .await;

        let mut client = SwitchClient::new(&mock_server.address().to_string())?;
        client.token = Some("test-token".to_string());

        client.patch_rstp_config("1", Some(true), Some(1)).await?;
        Ok(())
    }

    include!("tests_more.rs");
}
