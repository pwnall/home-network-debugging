use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use trendnet_teg3102ws_config::{CliConfig, LbdConfig, NetworkConfig, StpConfig, SwitchClient};

/// The command-line interface configuration for the TRENDnet switch utility.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the network configuration JSON file containing switch credentials.
    #[arg(short, long, default_value = "data/network.json")]
    config: PathBuf,

    /// The specific command to execute.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current system status (firmware, uptime, etc.)
    Status,
    /// Show/Set system settings (name, location, etc.)
    SystemSettings {
        /// Set System Name
        #[arg(long)]
        name: Option<String>,
        /// Set System Location
        #[arg(long)]
        location: Option<String>,
        /// Set System Contact
        #[arg(long)]
        contact: Option<String>,
    },
    /// Reboot the switch
    Reboot,
    /// Show the MAC address forwarding table
    MacTable,
    /// Show detailed port configuration
    Ports,
    /// Show STP configuration
    ShowStp,
    /// Enable or disable STP
    SetStp {
        /// Enable STP (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
        /// STP Protocol (mstp, rstp)
        #[arg(long)]
        protocol: Option<String>,
        /// Bridge Priority (0-61440, multiples of 4096)
        #[arg(long)]
        priority: Option<u32>,
    },
    /// Show RSTP per-port status
    RstpPorts,
    /// Show CIST per-port status
    CistPorts,
    /// Configure RSTP on one or more ports
    SetRstpPort {
        /// Port ID(s) (1-10, t1-t8). Comma-separated for multiple.
        #[arg(long)]
        id: String,
        /// Set as Edge Port (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        edge: Option<bool>,
        /// Set Path Cost
        #[arg(long)]
        path_cost: Option<u32>,
    },
    /// Show Loopback Detection configuration
    ShowLbd,
    /// Enable or disable Loopback Detection
    SetLbd {
        /// Enable LBD (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Show CLI (SSH/Telnet) configuration
    ShowCli,
    /// Enable or disable SSH/Telnet
    SetCli {
        /// Enable SSH
        #[arg(long)]
        ssh: bool,
        /// Enable Telnet
        #[arg(long)]
        telnet: bool,
    },
    /// Configure a port
    SetPort {
        /// Port ID (1-10, t1-t8)
        #[arg(long)]
        id: String,
        /// Enable/Disable port
        #[arg(long)]
        enable: Option<bool>,
        /// Set description
        #[arg(long)]
        description: Option<String>,
        /// Enable EAP Passthrough (true/false)
        #[arg(long)]
        eap_passthrough: Option<bool>,
        /// Set PVID
        #[arg(long)]
        pvid: Option<u32>,
        /// Set Link Speed Configuration
        #[arg(long)]
        speed: Option<String>,
    },
    /// Show IGMP snooping configuration
    ShowIgmp,
    /// Show Multicast Filter configuration
    ShowMulticastFilter,
    /// Show DHCP Snooping configuration
    ShowDhcpSnooping,
    /// Show Jumbo Frame configuration
    ShowJumboFrame,
    /// Show Storm Control configuration
    ShowStormControl,
    /// Show DoS protection configuration
    ShowDos,
    /// Show QoS configuration
    ShowQos,
    /// Show LLDP configuration
    ShowLldp,
    /// Show EEE configuration
    ShowEee,
    /// Set EEE configuration for a port
    SetEee {
        /// Port number
        #[arg(long)]
        port: u32,
        /// Enable EEE (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Set IGMP configuration
    SetIgmp {
        /// Enable IGMP snooping (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Set Multicast Filter configuration
    SetMulticastFilter {
        /// Enable Multicast Filter (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Set DHCP Snooping configuration
    SetDhcpSnooping {
        /// Enable DHCP snooping (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Set DHCP Trust status for a port
    SetDhcpTrust {
        /// Port ID
        #[arg(long)]
        id: String,
        /// Trust state (trusted/untrusted)
        #[arg(long)]
        state: String,
    },
    /// Set Jumbo Frame configuration
    SetJumboFrame {
        /// Frame size
        #[arg(long)]
        size: u32,
    },
    /// Set Storm Control configuration for a port
    SetStormControl {
        /// Port number
        #[arg(long)]
        port: u32,
        /// Broadcast rate (kbps)
        #[arg(long, default_value = "0")]
        broadcast: u64,
        /// Unknown multicast rate (kbps)
        #[arg(long, default_value = "0")]
        multicast: u64,
        /// Unknown unicast rate (kbps)
        #[arg(long, default_value = "0")]
        unicast: u64,
    },
    /// Set DoS protection configuration
    SetDos {
        /// Enable DoS protection (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Set QoS configuration
    SetQos {
        /// Enable QoS (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Set LLDP configuration
    SetLldp {
        /// Enable LLDP (true/false)
        #[arg(long, action = clap::ArgAction::Set)]
        enable: bool,
    },
    /// Save configuration to flash
    Save,
}

/// The main entry point for the TRENDnet switch configuration CLI.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_content = fs::read_to_string(&cli.config)?;
    let network_config: NetworkConfig = serde_json::from_str(&config_content)?;

    let mut client = SwitchClient::new(&network_config.switch.ip)?;
    client
        .login(
            &network_config.switch.username,
            &network_config.switch.password,
        )
        .await?;

    match cli.command {
        Commands::Status => {
            let status = client.get_system_status().await?;
            println!("SystemStatus {{");
            println!("    device_name: {:?},", status.device_name);
            println!("    firmware_version: {:?},", status.firmware_version);
            let vendor = oui_data::lookup(&status.mac_addr)
                .map(|r| r.organization())
                .unwrap_or("Unknown");
            println!("    mac_addr: {:?} ({}),", status.mac_addr, vendor);
            println!("    serial_no: {:?},", status.serial_no);
            println!("    model_name: {:?},", status.model_name);
            println!("    temperature: {},", status.temperature);
            println!("    up_time: {},", status.up_time);
            println!("}}");
        }
        Commands::SystemSettings {
            name,
            location,
            contact,
        } => {
            let mut settings = client.get_system_settings().await?;
            if name.is_none() && location.is_none() && contact.is_none() {
                println!("{:#?}", settings);
                return Ok(());
            }
            if let Some(n) = name {
                settings.device_name = n;
            }
            if let Some(l) = location {
                settings.system_location = l;
            }
            if let Some(c) = contact {
                settings.system_contact = c;
            }
            client.patch_system_settings(&settings).await?;
            println!("System settings updated.");
        }
        Commands::Reboot => {
            client.reboot().await?;
            println!("Reboot command sent.");
        }
        Commands::MacTable => {
            let table = client.get_mac_table().await?;
            println!(
                "{:<5} | {:<20} | {:<10} | {:<10} | Vendor",
                "VLAN", "MAC Address", "Port", "Mode"
            );
            println!(
                "{:-<5}-|-{:-<20}-|-{:-<10}-|-{:-<10}-|-{:-<30}",
                "", "", "", "", ""
            );
            for entry in table {
                let vendor = oui_data::lookup(&entry.mac_addr)
                    .map(|r| r.organization())
                    .unwrap_or("Unknown");
                println!(
                    "{:<5} | {:<20} | {:<10} | {:<10} | {}",
                    entry.vlan_id, entry.mac_addr, entry.port_id, entry.mode, vendor
                );
            }
        }
        Commands::Ports => {
            let ports = client.get_ports().await?;
            println!(
                "{:<5} | {:<7} | {:<10} | {:<5} | {:<10} | {:<15} | {:<5}",
                "Port", "Enabled", "Speed", "PVID", "EAP Pass", "Description", "MTU"
            );
            println!(
                "{:-<5}-|-{:-<7}-|-{:-<10}-|-{:-<5}-|-{:-<10}-|-{:-<15}-|-{:-<5}",
                "", "", "", "", "", "", ""
            );
            for port in ports {
                println!(
                    "{:<5} | {:<7} | {:<10} | {:<5} | {:<10} | {:<15} | {:<5}",
                    port.port_id,
                    port.enable,
                    port.link_speed.unwrap_or_default(),
                    port.pvid,
                    port.eap_passthrough,
                    port.description,
                    port.mtu
                );
            }
        }
        Commands::ShowStp => {
            let config = client.get_stp_config().await?;
            println!("{:#?}", config);
        }
        Commands::SetStp {
            enable,
            protocol,
            priority,
        } => {
            let config = StpConfig {
                enable,
                protocol,
                root_bridge: None,
                global_stp_settings: priority.map(|p| {
                    trendnet_teg3102ws_config::GlobalStpSettings {
                        priority: Some(p),
                        forward_delay: None,
                        maximum_age: None,
                        tx_hold_count: None,
                        hello_time: None,
                    }
                }),
                stp_port_counter: None,
            };
            client.patch_stp_config(&config).await?;
            println!("STP configuration updated.");
        }
        Commands::RstpPorts => match client.get_rstp_ports().await {
            Ok(ports) => {
                println!(
                    "{:<5} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10}",
                    "Port", "Role", "State", "Priority", "Path Cost", "Edge Port"
                );
                println!(
                    "{:-<5}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}",
                    "", "", "", "", "", ""
                );
                for port in ports {
                    println!(
                        "{:<5} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10}",
                        port.port_id,
                        port.port_role.unwrap_or_default(),
                        port.port_state.unwrap_or_default(),
                        port.priority,
                        port.path_cost,
                        port.edge_port_conf
                    );
                }
            }
            Err(e) => {
                if e.to_string().contains("2715") {
                    println!("Switch is in MSTP mode. Use 'cist-ports' instead.");
                } else {
                    return Err(e);
                }
            }
        },
        Commands::CistPorts => match client.get_cist_ports().await {
            Ok(ports) => {
                println!(
                    "{:<5} | {:<10} | {:<10} | {:<10} | {:<10}",
                    "Port", "Role", "State", "Priority", "Path Cost"
                );
                println!(
                    "{:-<5}-|-{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}",
                    "", "", "", "", ""
                );
                for port in ports {
                    println!(
                        "{:<5} | {:<10} | {:<10} | {:<10} | {:<10}",
                        port.port_id,
                        port.port_role.unwrap_or_default(),
                        port.port_state.unwrap_or_default(),
                        port.priority,
                        port.path_cost
                    );
                }
            }
            Err(e) => {
                if !e.to_string().contains("6004") {
                    return Err(e);
                }
                println!("Switch is in RSTP mode. Use 'rstp-ports' instead.");
            }
        },
        Commands::SetRstpPort {
            id,
            edge,
            path_cost,
        } => {
            let ids: Vec<&str> = id.split(',').map(|s| s.trim()).collect();
            for port_id in ids {
                client.patch_rstp_config(port_id, edge, path_cost).await?;
                println!("Port {} RSTP configuration updated.", port_id);
            }
        }
        Commands::ShowLbd => {
            let config = client.get_lbd_config().await?;
            println!("{:#?}", config);
        }
        Commands::SetLbd { enable } => {
            let config = LbdConfig { enable };
            client.patch_lbd_config(&config).await?;
            println!("LBD configuration updated.");
        }
        Commands::ShowCli => {
            let config = client.get_cli_config().await?;
            println!("{:#?}", config);
        }
        Commands::SetCli { ssh, telnet } => {
            let config = CliConfig {
                ssh_enable: ssh,
                telnet_enable: telnet,
            };
            client.patch_cli_config(&config).await?;
            println!("CLI configuration updated.");
        }
        Commands::SetPort {
            id,
            enable,
            description,
            eap_passthrough,
            pvid,
            speed,
        } => {
            let ports = client.get_ports().await?;
            let mut port = ports
                .into_iter()
                .find(|p| p.port_id == id)
                .ok_or_else(|| anyhow::anyhow!("Port not found"))?;

            if let Some(e) = enable {
                port.enable = e;
            }
            if let Some(d) = description {
                port.description = d;
            }
            if let Some(ep) = eap_passthrough {
                port.eap_passthrough = ep;
            }
            if let Some(p) = pvid {
                port.pvid = p;
            }
            if let Some(s) = speed {
                port.link_speed_conf = s;
            }

            client.patch_port_config(&port).await?;
            println!("Port {} configuration updated.", id);
        }
        Commands::ShowIgmp => {
            let config = client.get_igmp_config().await?;
            println!("{:#?}", config);
        }
        Commands::ShowMulticastFilter => {
            let config = client.get_multicast_filter_config().await?;
            println!("{:#?}", config);
        }
        Commands::ShowDhcpSnooping => {
            let config = client.get_dhcp_snooping_config().await?;
            println!("{:#?}", config);
            let trust_ports = client.get_dhcp_snooping_trust_ports().await?;
            println!("Trust Ports: {:#?}", trust_ports);
        }
        Commands::ShowJumboFrame => {
            let config = client.get_jumbo_frame_config().await?;
            println!("{:#?}", config);
        }
        Commands::ShowStormControl => {
            let config = client.get_storm_control_config().await?;
            println!(
                "{:<5} | {:<10} | {:<20} | {:<20}",
                "Port", "Broadcast", "Unknown Multicast", "Unknown Unicast"
            );
            println!("{:-<5}-|-{:-<10}-|-{:-<20}-|-{:-<20}", "", "", "", "");
            for entry in config {
                println!(
                    "{:<5} | {:<10} | {:<20} | {:<20}",
                    entry.port_no, entry.broadcast, entry.unknown_multicast, entry.unknown_unicast
                );
            }
        }
        Commands::ShowDos => {
            let config = client.get_dos_config().await?;
            println!("{:#?}", config);
        }
        Commands::ShowQos => {
            let config = client.get_qos_config().await?;
            println!("{:#?}", config);
        }
        Commands::ShowLldp => {
            let config = client.get_lldp_config().await?;
            println!("{:#?}", config);
        }
        Commands::ShowEee => {
            let config = client.get_eee_config().await?;
            println!("{:<5} | {:<10}", "Port", "Enabled");
            println!("{:-<5}-|-{:-<10}", "", "");
            for entry in config {
                println!("{:<5} | {:<10}", entry.port_no, entry.enable);
            }
        }
        Commands::SetEee { port, enable } => {
            client.patch_eee_config(port, enable).await?;
            println!("Port {} EEE configuration updated.", port);
        }
        Commands::SetIgmp { enable } => {
            let config = trendnet_teg3102ws_config::IgmpConfig {
                enable,
                report_suppression: 5,
            };
            client.patch_igmp_config(&config).await?;
            println!("IGMP configuration updated.");
        }
        Commands::SetMulticastFilter { enable } => {
            let config = trendnet_teg3102ws_config::MulticastFilterConfig { enable };
            client.patch_multicast_filter_config(&config).await?;
            println!("Multicast Filter configuration updated.");
        }
        Commands::SetDhcpSnooping { enable } => {
            let config = trendnet_teg3102ws_config::DhcpSnoopingConfig {
                enable,
                mac_verify: false,
                vlan: vec![trendnet_teg3102ws_config::DhcpSnoopingVlan { enable, vlan_id: 1 }],
            };
            client.patch_dhcp_snooping_config(&config).await?;
            println!("DHCP Snooping configuration updated.");
        }
        Commands::SetDhcpTrust { id, state } => {
            client.patch_dhcp_snooping_trust_ports(&id, &state).await?;
            println!("Port {} DHCP Trust state updated to {}.", id, state);
        }
        Commands::SetJumboFrame { size } => {
            client.patch_jumbo_frame_config(size).await?;
            println!("Jumbo Frame size updated to {}.", size);
        }
        Commands::SetStormControl {
            port,
            broadcast,
            multicast,
            unicast,
        } => {
            client
                .patch_storm_control_config(port, broadcast, multicast, unicast)
                .await?;
            println!("Port {} Storm Control configuration updated.", port);
        }
        Commands::SetDos { enable } => {
            client.patch_dos_config(enable).await?;
            println!("DoS protection configuration updated.");
        }
        Commands::SetQos { enable } => {
            let config = trendnet_teg3102ws_config::QosConfig {
                enable,
                schedule_method: None,
                trust_mode: None,
            };
            client.patch_qos_config(&config).await?;
            println!("QoS configuration updated.");
        }
        Commands::SetLldp { enable } => {
            client.patch_lldp_config(enable).await?;
            println!("LLDP configuration updated.");
        }
        Commands::Save => {
            client.save_config().await?;
            println!("Configuration saved to flash.");
        }
    }

    Ok(())
}
