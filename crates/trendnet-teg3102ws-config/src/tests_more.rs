#[tokio::test]
async fn test_patch_system_settings() -> Result<()> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("PATCH"))
        .and(path("/api/system/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let settings = SystemSettings {
        device_name: "test".into(),
        system_description: "desc".into(),
        system_location: "loc".into(),
        system_contact: "contact".into(),
    };
    client.patch_system_settings(&settings).await?;
    Ok(())
}

#[tokio::test]
async fn test_save_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/api/system/save"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    client.save_config().await?;
    Ok(())
}

#[tokio::test]
async fn test_get_stp_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/stp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "stpConfs": {
                    "enable": true,
                    "protocol": "rstp"
                },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_stp_config().await?;
    assert!(config.enable);
    assert_eq!(config.protocol.unwrap(), "rstp");
    Ok(())
}

#[tokio::test]
async fn test_patch_stp_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/stp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = StpConfig { enable: true, protocol: Some("rstp".into()), root_bridge: None, global_stp_settings: None, stp_port_counter: None };
    client.patch_stp_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_get_rstp_ports() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/stp/rstp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "rstpPortConfs": [{
                    "portID": "1",
                    "priority": 128,
                    "pathCost": 100,
                    "edgePortConf": "Yes"
                }],
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let ports = client.get_rstp_ports().await?;
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].port_id, "1");
    Ok(())
}

#[tokio::test]
async fn test_get_cist_ports() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/stp/cist"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "cistPortConfs": [{
                    "portID": "1",
                    "priority": 128,
                    "pathCost": 100
                }],
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let ports = client.get_cist_ports().await?;
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].port_id, "1");
    Ok(())
}

#[tokio::test]
async fn test_lbd_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/lbd"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "lbdConfs": { "enable": true },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/lbd"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_lbd_config().await?;
    assert!(config.enable);
    client.patch_lbd_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_cli_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/system/settings/session/cli"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "webConfs": { "sshEnable": true, "telnetEnable": false },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/session/cli"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_cli_config().await?;
    assert!(config.ssh_enable);
    client.patch_cli_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_igmp_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/igmp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "igmpConfs": { "enable": true, "reportSuppression": 5 },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/igmp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_igmp_config().await?;
    assert!(config.enable);
    client.patch_igmp_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_multicast_filter_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/multicastfilter"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "mcastFilterConfs": { "enable": true },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/multicastfilter"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_multicast_filter_config().await?;
    assert!(config.enable);
    client.patch_multicast_filter_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_dhcp_snooping_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/system/settings/dhcpsnp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "dhcpsnpConfs": { "enable": true, "macVerify": false, "vlan": [{"enable": true, "vlanID": 1}] },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/dhcpsnp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_dhcp_snooping_config().await?;
    assert!(config.enable);
    client.patch_dhcp_snooping_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_dhcp_snooping_trust_ports() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/system/settings/dhcpsnp/trustports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "portStatus": [{"portID": "1", "State": "trusted", "Action": "drop"}],
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/dhcpsnp/trustports"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_dhcp_snooping_trust_ports().await?;
    assert_eq!(config.len(), 1);
    client.patch_dhcp_snooping_trust_ports("1", "trusted").await?;
    Ok(())
}

#[tokio::test]
async fn test_storm_control_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/ports/stormcontrol"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "stormCtrlConfs": [{"portNo": 1, "broadcast": 0, "unknownMulticast": 0, "unknownUnicast": 0}],
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/ports/stormcontrol"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_storm_control_config().await?;
    assert_eq!(config.len(), 1);
    client.patch_storm_control_config(1, 0, 0, 0).await?;
    Ok(())
}

#[tokio::test]
async fn test_dos_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/system/settings/dos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "dosConfs": { "enable": true },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/dos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_dos_config().await?;
    assert!(config.enable);
    client.patch_dos_config(true).await?;
    Ok(())
}

#[tokio::test]
async fn test_qos_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/system/settings/qos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "qosConfs": { "enable": true },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/qos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_qos_config().await?;
    assert!(config.enable);
    client.patch_qos_config(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_lldp_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/lldp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "lldpGlobalConfs": { "enable": true, "holdtime": 4, "reinitDelay": 2, "transmitDelay": 2, "transmitInterval": 30, "version": 2 },
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/lldp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_lldp_config().await?;
    assert!(config.enable);
    client.patch_lldp_config(true).await?;
    Ok(())
}

#[tokio::test]
async fn test_eee_config() -> Result<()> {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/ports/eee"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "portEeeConfs": [{"enable": true, "portNo": 1}],
                "errCode": 0
            }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/api/ports/eee_extended"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": { "errCode": 0, "message": "OK" }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let config = client.get_eee_config().await?;
    assert_eq!(config.len(), 1);
    client.patch_eee_config(1, true).await?;
    Ok(())
}

#[tokio::test]
async fn test_get_mgmt_interface() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/system/settings/mgmtinterface"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "mgmtInterface": {
                    "IP": "192.168.10.200",
                    "configuration": "static",
                    "defaultGateway": "",
                    "dhcpOption43": "",
                    "dns1IP": "0.0.0.0",
                    "dns2IP": "0.0.0.0",
                    "submask": "255.255.255.0",
                    "uplinkPort": "0",
                    "vlanID": 1
                },
                "errCode": 0,
                "message": "OK"
            }
        })))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());

    let mgmt = client.get_mgmt_interface().await?;
    assert_eq!(mgmt.ip, "192.168.10.200");
    assert_eq!(mgmt.submask, "255.255.255.0");
    Ok(())
}

#[tokio::test]
async fn test_patch_mgmt_interface() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/mgmtinterface"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "restful_res": {
                "errCode": 0,
                "message": "OK"
            }
        })))
        .mount(&mock_server)
        .await;
        
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut client = SwitchClient::new(&mock_server.address().to_string())?;
    client.token = Some("test-token".to_string());
    
    let mock_addr = mock_server.address().to_string();
    let new_ip = mock_addr;
    
    let config = MgmtInterface {
        ip: new_ip,
        configuration: "static".to_string(),
        default_gateway: "".to_string(),
        dhcp_option_43: "".to_string(),
        dns1_ip: "0.0.0.0".to_string(),
        dns2_ip: "0.0.0.0".to_string(),
        submask: "255.255.255.0".to_string(),
        uplink_port: "0".to_string(),
        vlan_id: 1,
    };

    client.patch_mgmt_interface(&config).await?;
    Ok(())
}

#[tokio::test]
async fn test_change_password_first_login() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/account"))
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

    client
        .change_password("admin", "newpassword123", true, None)
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_change_password_subsequent_login() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/system/settings/account"))
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

    client
        .change_password("admin", "newpassword123", false, Some("oldpassword456"))
        .await?;
    Ok(())
}