use anyhow::Result;
use clap::Parser;
use nest_wifi_pro_ethernet::{format_mac, NestScanner, PacketType};
use std::path::PathBuf;

/// The command-line interface configuration for the Nest WiFi Pro packet analyzer.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to a pcap file to analyze.
    #[arg(short, long)]
    pcap: Option<PathBuf>,

    /// Network interface to capture from live.
    #[arg(short, long)]
    interface: Option<String>,

    /// Capture duration in seconds (for live capture).
    #[arg(short, long, default_value = "65")]
    duration: u64,

    /// Log details of MAC addresses seen on the network.
    #[arg(long)]
    log_details: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut scanner = NestScanner::new();

    if let Some(pcap_path) = cli.pcap {
        let mut cap = pcap::Capture::from_file(pcap_path)?;
        while let Ok(packet) = cap.next_packet() {
            scanner.process_packet(packet.data, cli.log_details);
        }
    } else if let Some(interface_name) = cli.interface {
        let cap = pcap::Capture::from_device(interface_name.as_str())?
            .promisc(true)
            .snaplen(65535)
            .immediate_mode(true)
            .timeout(100)
            .open()?;

        let mut cap = cap.setnonblock()?;

        println!(
            "Capturing on {} for {} seconds...",
            interface_name, cli.duration
        );
        let start = std::time::Instant::now();
        let mut seen_macs = std::collections::HashSet::new();
        while start.elapsed().as_secs() < cli.duration {
            match cap.next_packet() {
                Ok(packet) => {
                    if packet.data.len() >= 12 {
                        let src_mac = &packet.data[6..12];
                        let mac_str = format!(
                            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                            src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5]
                        );
                        if seen_macs.insert(mac_str.clone()) {
                            let vendor = oui_data::lookup(&mac_str)
                                .map(|r| r.organization())
                                .unwrap_or("Unknown");
                            println!("DEBUG: Seen source MAC: {} ({})", mac_str, vendor);
                        }
                    }
                    scanner.process_packet(packet.data, cli.log_details);
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // Should only happen in blocking mode, but harmless
                }
                Err(_) => {
                    // In non-blocking mode, this is typically TimeoutExpired or WouldBlock.
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    } else {
        println!("Please specify either a --pcap file or an --interface. Use -h for help.");
        return Ok(());
    }

    println!("\nNest WiFi Pro Device Analysis:");

    let mut devices: Vec<_> = scanner.devices.values().collect();
    devices.sort_by_key(|d| d.mac);

    for device in devices {
        let mac_str = format_mac(&device.mac);
        let vendor = oui_data::lookup(&mac_str)
            .map(|r| r.organization())
            .unwrap_or("Unknown");
        println!("\nDevice: {} ({})", mac_str, vendor);

        let stp_status = if device.seen_packets.contains(&PacketType::Stp) {
            "yes"
        } else {
            "no"
        };
        print!("  STP:        {:<5}", stp_status);
        if let Some(ref info) = device.stp {
            print!(
                " (Protocol: {}, Root: {} [Prio: {}], Bridge: {} [Prio: {}], Cost: {})",
                info.protocol,
                info.root_id,
                info.root_priority,
                info.bridge_id,
                info.bridge_priority,
                info.root_path_cost
            );
        }
        println!();

        let dhcp_status = if device.seen_packets.contains(&PacketType::DhcpRouge) {
            "yes"
        } else {
            "no"
        };
        print!("  DHCP Rouge: {:<5}", dhcp_status);
        if let Some(ref info) = device.dhcp {
            print!(" (XID: 0x{:08x})", info.xid);
        }
        println!();

        let mdns_status = if device.seen_packets.contains(&PacketType::Mdns) {
            "yes"
        } else {
            "no"
        };
        print!("  mDNS:       {:<5}", mdns_status);
        if !device.mdns_services.is_empty() {
            let services: Vec<_> = device.mdns_services.iter().cloned().collect();
            print!(" ({})", services.join(", "));
        }
        println!();

        let ipv6_status = if device.seen_packets.contains(&PacketType::Ipv6Ra) {
            "yes"
        } else {
            "no"
        };
        print!("  IPv6 RA:    {:<5}", ipv6_status);
        if let Some(ref info) = device.ipv6 {
            print!(" (Source: {})", info.source);
        }
        println!();

        let ssdp_status = if device.seen_packets.contains(&PacketType::Ssdp) {
            "yes"
        } else {
            "no"
        };
        print!("  SSDP:       {:<5}", ssdp_status);
        if let Some(ref info) = device.ssdp {
            print!(" (Server: {})", info.server);
        }
        println!();

        let arp_status = if device.seen_packets.contains(&PacketType::Arp) {
            "yes"
        } else {
            "no"
        };
        print!("  ARP:        {:<5}", arp_status);
        if let Some(ref info) = device.arp {
            print!(" (IP: {})", info.sender_ip);
        }
        println!();
    }

    Ok(())
}
