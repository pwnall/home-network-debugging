use etherparse::SlicedPacket;
use std::collections::{HashMap, HashSet};

/// Represents the types of network packets monitored from Nest WiFi Pro devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketType {
    /// Spanning Tree Protocol packets (e.g., 802.1D, RSTP).
    Stp,
    /// Custom DHCP Discover packets used for Google Nest rogue detection.
    DhcpRouge,
    /// Multicast DNS packets for service discovery.
    Mdns,
    /// ICMPv6 Router Advertisement packets.
    Ipv6Ra,
    /// Simple Service Discovery Protocol packets.
    Ssdp,
    /// Address Resolution Protocol packets.
    Arp,
}

impl PacketType {
    /// Returns a list of all available packet types.
    pub fn all() -> Vec<Self> {
        vec![
            PacketType::Stp,
            PacketType::DhcpRouge,
            PacketType::Mdns,
            PacketType::Ipv6Ra,
            PacketType::Ssdp,
            PacketType::Arp,
        ]
    }

    /// Returns a human-readable string representation of the packet type.
    pub fn as_str(&self) -> &'static str {
        match self {
            PacketType::Stp => "STP",
            PacketType::DhcpRouge => "DHCP Rouge",
            PacketType::Mdns => "mDNS",
            PacketType::Ipv6Ra => "IPv6 RA",
            PacketType::Ssdp => "SSDP",
            PacketType::Arp => "ARP",
        }
    }
}

/// Contains information extracted from Spanning Tree Protocol (STP) packets.
#[derive(Debug, Default, Clone)]
pub struct StpInfo {
    /// The unique identifier of the root bridge.
    pub root_id: String,
    /// The priority value of the root bridge.
    pub root_priority: u16,
    /// The unique identifier of the bridge transmitting the packet.
    pub bridge_id: String,
    /// The priority value of the transmitting bridge.
    pub bridge_priority: u16,
    /// The advertised cost to reach the root bridge.
    pub root_path_cost: u32,
    /// The specific STP protocol version detected.
    pub protocol: String,
}

/// Contains information extracted from specialized DHCP rogue detection packets.
#[derive(Debug, Default, Clone)]
pub struct DhcpInfo {
    /// The transaction ID (XID) from the DHCP packet.
    pub xid: u32,
}

/// Contains information extracted from IPv6 Router Advertisement packets.
#[derive(Debug, Default, Clone)]
pub struct IPv6Info {
    /// The source IPv6 address (typically link-local).
    pub source: String,
}

/// Contains information extracted from SSDP discovery packets.
#[derive(Debug, Default, Clone)]
pub struct SsdpInfo {
    /// The server string advertised.
    pub server: String,
}

/// Contains information extracted from ARP packets.
#[derive(Debug, Default, Clone)]
pub struct ArpInfo {
    /// The sender IP address.
    pub sender_ip: String,
    /// The target IP address.
    pub target_ip: String,
}

/// Represents a single Nest WiFi Pro device detected on the network.
#[derive(Debug, Default)]
pub struct NestDevice {
    /// The MAC address of the device.
    pub mac: [u8; 6],
    /// The set of packet types that have been observed originating from this device.
    pub seen_packets: HashSet<PacketType>,
    /// Extracted STP information, if observed.
    pub stp: Option<StpInfo>,
    /// Extracted DHCP information, if observed.
    pub dhcp: Option<DhcpInfo>,
    /// Extracted IPv6 information, if observed.
    pub ipv6: Option<IPv6Info>,
    /// A set of identified mDNS services advertised by this device.
    pub mdns_services: HashSet<String>,
    /// Extracted SSDP information, if observed.
    pub ssdp: Option<SsdpInfo>,
    /// Extracted ARP information, if observed.
    pub arp: Option<ArpInfo>,
}

/// A scanner that processes raw network packets and builds a profile of detected Nest devices.
pub struct NestScanner {
    /// A mapping of MAC addresses to detected Nest devices.
    pub devices: HashMap<[u8; 6], NestDevice>,
}

impl Default for NestScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl NestScanner {
    /// Creates a new, empty `NestScanner`.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Processes a raw Ethernet packet, extracting relevant information if it originates
    /// from or relates to a Nest WiFi Pro device.
    pub fn process_packet(&mut self, data: &[u8], log_details: bool) {
        let Ok(value) = SlicedPacket::from_ethernet(data) else {
            return;
        };
        let src_mac = match value.link {
            Some(etherparse::LinkSlice::Ethernet2(ref h)) => h.source(),
            _ => return,
        };

        let stp_info = Self::check_stp(data);
        let dhcp_info = Self::check_dhcp_rouge(&value);
        let mdns_services = Self::check_mdns(&value);
        let ipv6_info = Self::check_ipv6_ra(&value);
        let ssdp_info = Self::check_ssdp(&value);
        let arp_info = Self::check_arp(data);

        let is_google_src = Self::has_google_source_mac(&src_mac);
        let is_google_stp = stp_info.as_ref().is_some_and(|info| {
            // Check if Root ID or Bridge ID starts with Google OUI
            // Root ID hex string starts with priority (4 chars) then MAC
            info.root_id.len() >= 10 && &info.root_id[4..10] == "b87bd4"
        });

        if !is_google_src && !is_google_stp {
            return;
        }

        let device = self.devices.entry(src_mac).or_insert_with(|| NestDevice {
            mac: src_mac,
            ..Default::default()
        });

        if let Some(ref info) = stp_info {
            device.seen_packets.insert(PacketType::Stp);
            device.stp = Some(info.clone());
            if log_details {
                println!(
                    "DEBUG: [STP] Source: {} - Protocol: {}, Root: {}, Bridge: {}",
                    format_mac(&src_mac),
                    info.protocol,
                    info.root_id,
                    info.bridge_id
                );
            }
        }

        // Only process other types if the source itself is Google
        if !is_google_src {
            return;
        }

        if let Some(ref info) = dhcp_info {
            device.seen_packets.insert(PacketType::DhcpRouge);
            device.dhcp = Some(info.clone());
            if log_details {
                println!(
                    "DEBUG: [DHCP Rouge] Source: {} - XID: 0x{:08x}",
                    format_mac(&src_mac),
                    info.xid
                );
            }
        }
        if !mdns_services.is_empty() {
            device.seen_packets.insert(PacketType::Mdns);
            device.mdns_services.extend(mdns_services.clone());
            if log_details {
                let services: Vec<_> = mdns_services.iter().cloned().collect();
                println!(
                    "DEBUG: [mDNS] Source: {} - Services: {}",
                    format_mac(&src_mac),
                    services.join(", ")
                );
            }
        }
        if let Some(ref info) = ipv6_info {
            device.seen_packets.insert(PacketType::Ipv6Ra);
            device.ipv6 = Some(info.clone());
            if log_details {
                println!(
                    "DEBUG: [IPv6 RA] Source: {} - IPv6: {}",
                    format_mac(&src_mac),
                    info.source
                );
            }
        }
        if let Some(ref info) = ssdp_info {
            device.seen_packets.insert(PacketType::Ssdp);
            device.ssdp = Some(info.clone());
            if log_details {
                println!(
                    "DEBUG: [SSDP] Source: {} - Server: {}",
                    format_mac(&src_mac),
                    info.server
                );
            }
        }
        if let Some(ref info) = arp_info {
            device.seen_packets.insert(PacketType::Arp);
            device.arp = Some(info.clone());
            if log_details {
                println!(
                    "DEBUG: [ARP] Source: {} - Sender IP: {}, Target IP: {}",
                    format_mac(&src_mac),
                    info.sender_ip,
                    info.target_ip
                );
            }
        }
    }

    /// Checks if a packet is an SSDP discovery packet on port 1900 and extracts the server.
    fn check_ssdp(value: &SlicedPacket) -> Option<SsdpInfo> {
        let Some(etherparse::TransportSlice::Udp(ref udp)) = value.transport else {
            return None;
        };
        if udp.destination_port() == 1900 {
            let payload = value.payload;
            if let Ok(text) = std::str::from_utf8(payload) {
                let mut server = "Unknown".to_string();
                for line in text.lines() {
                    let line_upper = line.to_uppercase();
                    if line_upper.starts_with("SERVER:") {
                        server = line[7..].trim().to_string();
                        break;
                    }
                }
                return Some(SsdpInfo { server });
            }
        }
        None
    }

    /// Checks if a packet is an ARP packet based on its EtherType and extracts IPs.
    fn check_arp(data: &[u8]) -> Option<ArpInfo> {
        if data.len() >= 42 {
            let eth_type_len = u16::from_be_bytes([data[12], data[13]]);
            if eth_type_len == 0x0806 {
                let sender_ip = format!("{}.{}.{}.{}", data[28], data[29], data[30], data[31]);
                let target_ip = format!("{}.{}.{}.{}", data[38], data[39], data[40], data[41]);
                return Some(ArpInfo {
                    sender_ip,
                    target_ip,
                });
            }
        }
        None
    }

    /// Extracts STP BPDU details from an LLC packet if present.
    fn check_stp(data: &[u8]) -> Option<StpInfo> {
        if data.len() >= 16 {
            let eth_type_len = u16::from_be_bytes([data[12], data[13]]);
            if eth_type_len <= 1500 && data[14] == 0x42 && data[15] == 0x42 {
                // STP BPDU starts at offset 17 (14 eth + 3 LLC)
                if data.len() >= 17 + 35 {
                    let protocol = if data[17..19] == [0, 0] {
                        if data[20] == 0x00 {
                            "802.1D"
                        } else if data[20] == 0x02 {
                            "802.1w (RSTP)"
                        } else {
                            "STP"
                        }
                    } else {
                        "Unknown"
                    };

                    let root_priority = u16::from_be_bytes([data[22], data[23]]);
                    let root_id = hex::encode(&data[22..30]);
                    let root_path_cost =
                        u32::from_be_bytes([data[30], data[31], data[32], data[33]]);
                    let bridge_priority = u16::from_be_bytes([data[34], data[35]]);
                    let bridge_id = hex::encode(&data[34..42]);

                    return Some(StpInfo {
                        root_id,
                        root_priority,
                        bridge_id,
                        bridge_priority,
                        root_path_cost,
                        protocol: protocol.to_string(),
                    });
                }
            }
        }
        None
    }

    /// Checks if the provided MAC address belongs to known Google OUIs.
    fn has_google_source_mac(mac: &[u8; 6]) -> bool {
        let mac_str = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );

        if let Some(record) = oui_data::lookup(&mac_str) {
            record.organization().contains("Google")
        } else {
            false
        }
    }

    /// Checks if a UDP packet is a DHCP Discover containing the "gwifi_rouge_dhcp_detection" string.
    fn check_dhcp_rouge(value: &SlicedPacket) -> Option<DhcpInfo> {
        if let Some(etherparse::TransportSlice::Udp(ref udp)) = value.transport {
            let sport = udp.source_port();
            let dport = udp.destination_port();

            if (sport == 67 || sport == 68) && (dport == 67 || dport == 68) {
                let payload = value.payload;
                if payload.len() >= 44 + 26 {
                    let sname = &payload[44..44 + 26];
                    if sname.starts_with(b"gwifi_rouge_dhcp_detection") {
                        let xid =
                            u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                        return Some(DhcpInfo { xid });
                    }
                }
            }
        }
        None
    }

    /// Identifies specific mDNS services advertised by the device.
    fn check_mdns(value: &SlicedPacket) -> HashSet<String> {
        let mut services = HashSet::new();
        if let Some(etherparse::TransportSlice::Udp(ref udp)) = value.transport {
            if udp.destination_port() == 5353 {
                let payload = value.payload;
                let known_services = [
                    ("_googlecast", "Google Cast"),
                    ("_matter", "Matter"),
                    ("_ipp", "IPP"),
                    ("_googlezone", "Google Zone"),
                    ("_sleep-proxy", "Sleep Proxy"),
                ];
                for (pattern, label) in &known_services {
                    if payload
                        .windows(pattern.len())
                        .any(|window| window == pattern.as_bytes())
                    {
                        services.insert(label.to_string());
                    }
                }
                // If we didn't match a known service but it's 5353, still return something
                if services.is_empty() {
                    services.insert("Other mDNS".to_string());
                }
            }
        }
        services
    }

    /// Checks if an ICMPv6 packet is a Router Advertisement and extracts the source address.
    fn check_ipv6_ra(value: &SlicedPacket) -> Option<IPv6Info> {
        if let Some(etherparse::TransportSlice::Icmpv6(ref icmp)) = value.transport {
            if icmp.type_u8() == 134 {
                if let Some(etherparse::InternetSlice::Ipv6(ref ip6, _)) = value.ip {
                    let source = std::net::Ipv6Addr::from(ip6.source());
                    return Some(IPv6Info {
                        source: source.to_string(),
                    });
                }
            }
        }
        None
    }
}

/// Formats a 6-byte MAC address into a standard colon-separated string.
pub fn format_mac(mac: &[u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_google_source_mac() {
        // Nest WiFi Pro OUI
        assert!(NestScanner::has_google_source_mac(&[
            0xb8, 0x7b, 0xd4, 0xa8, 0x2b, 0x1d
        ]));
        // Other Google OUIs seen on network
        assert!(NestScanner::has_google_source_mac(&[
            0x38, 0x86, 0xf7, 0x00, 0x00, 0x00
        ]));
        assert!(NestScanner::has_google_source_mac(&[
            0xdc, 0xe5, 0x5b, 0x00, 0x00, 0x00
        ]));
        assert!(NestScanner::has_google_source_mac(&[
            0xf0, 0xef, 0x86, 0x00, 0x00, 0x00
        ]));
        // Non-Google MAC (e.g. Dreame Technology)
        assert!(!NestScanner::has_google_source_mac(&[
            0x00, 0xae, 0xf7, 0x60, 0xf2, 0xe9
        ]));
        // Completely random unknown MAC
        assert!(!NestScanner::has_google_source_mac(&[
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55
        ]));
    }

    #[test]
    fn test_check_stp() {
        let hex_stp = "0180c2000000b87bd4a82b1d002642420300000000007900b87bd4a843e7000000647a00b87bd4a82b1e800300020600020003000000000000000000";
        let data = hex::decode(hex_stp).unwrap();
        let info = NestScanner::check_stp(&data).unwrap();
        assert_eq!(info.protocol, "802.1D");
        assert_eq!(info.root_id, "7900b87bd4a843e7");
    }

    #[test]
    fn test_check_dhcp_rouge() {
        let hex_dhcp = "ffffffffffffb87bd4a843e7080045000110000000000211b7de00000000ffffffff0044004300fc5fb2010106006a1290560001800000000000000000000000000000000000b87bd4a843e70000000000000000000067776966695f726f7567655f646863705f646574656374696f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063825363350101ff";
        let data = hex::decode(hex_dhcp).unwrap();
        let value = SlicedPacket::from_ethernet(&data).unwrap();
        let info = NestScanner::check_dhcp_rouge(&value).unwrap();
        assert_eq!(info.xid, 0x6a129056);
    }

    #[test]
    fn test_check_mdns() {
        let hex_mdns = "01005e0000fbb87bd4a843e708004500008d14c84000ff116ef2c0a85601e00000fb14e914e90079a3af00008400000000020000000021453030343839453542374638463836322d30303030303030303939354444433532075f6d6174746572045f746370056c6f63616c00002180010000007800150000000015a40c334537413441353235324546c03bc05200018001000000780004c0a85601";
        let data = hex::decode(hex_mdns).unwrap();
        let value = SlicedPacket::from_ethernet(&data).unwrap();
        let services = NestScanner::check_mdns(&value);
        assert!(!services.is_empty());
        assert!(services.contains("Matter"));
    }

    #[test]
    fn test_check_ipv6_ra() {
        // Truncated but valid headers for detection
        let hex_ipv6 = "333300000001b87bd4a843e786dd60079f1800203afffe80000000000000ba7bd4fffea843e7ff0200000000000000000000000000018600f3a840420e1000000000000000011802401800000708fd962ac7e0970001";
        let data = hex::decode(hex_ipv6).unwrap();
        let value = SlicedPacket::from_ethernet(&data).unwrap();
        let info = NestScanner::check_ipv6_ra(&value).unwrap();
        assert!(info.source.contains("fe80:"));
    }

    #[test]
    fn test_check_ssdp() {
        // Hex dump of an SSDP packet containing SERVER: Debian/rodete...
        let hex_ssdp = "01005e7ffffab87bd4a843e70800450001981405400002115cacc0a85601effffffab691076c0184dcc14e4f54494659202a20485454502f312e310d0a484f53543a203233392e3235352e3235352e3235303a313930300d0a43414348452d434f4e54524f4c3a206d61782d6167653d3132300d0a4c4f434154494f4e3a20687474703a2f2f3139322e3136382e38362e313a353030302f726f6f74446573632e786d6c0d0a5345525645523a2044656269616e2f726f646574652055506e502f312e31204d696e6955506e50642f312e390d0a4e543a2075706e703a726f6f746465766963650d0a55534e3a20757569643a36623665373563352d316636342d343231372d393362332d3362316164363137633533383a3a75706e703a726f6f746465766963650d0a4e54533a20737364703a616c6976650d0a4f50543a2022687474703a2f2f736368656d61732e75706e702e6f72672f75706e702f312f302f223b206e733d30310d0a30312d4e4c533a20310d0a424f4f5449442e55504e502e4f52473a20310d0a434f4e46494749442e55504e502e4f52473a20313333370d0a0d0a";
        let data = hex::decode(hex_ssdp).unwrap();
        let value = SlicedPacket::from_ethernet(&data).unwrap();
        let info = NestScanner::check_ssdp(&value).unwrap();
        assert_eq!(info.server, "Debian/rodete UPnP/1.1 MiniUPnPd/1.9");
    }

    #[test]
    fn test_check_arp() {
        let hex_arp = "ffffffffffffb87bd4a843e708060001080006040001b87bd4a843e7c0a85601000000000000c0a8561c00000000000000000000000000000000";
        let data = hex::decode(hex_arp).unwrap();
        let info = NestScanner::check_arp(&data).unwrap();
        assert_eq!(info.sender_ip, "192.168.86.1");
        assert_eq!(info.target_ip, "192.168.86.28");
    }

    #[test]
    fn test_process_packet_full() {
        let mut scanner = NestScanner::new();

        let stp_hex = "0180c2000000b87bd4a82b1d002642420300000000007900b87bd4a843e7000000647a00b87bd4a82b1e800300020600020003000000000000000000";
        scanner.process_packet(&hex::decode(stp_hex).unwrap(), false);

        let dhcp_hex = "ffffffffffffb87bd4a843e7080045000110000000000211b7de00000000ffffffff0044004300fc5fb2010106006a1290560001800000000000000000000000000000000000b87bd4a843e70000000000000000000067776966695f726f7567655f646863705f646574656374696f6e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063825363350101ff";
        scanner.process_packet(&hex::decode(dhcp_hex).unwrap(), false);

        assert_eq!(scanner.devices.len(), 2);

        let stp_dev = scanner
            .devices
            .get(&[0xb8, 0x7b, 0xd4, 0xa8, 0x2b, 0x1d])
            .unwrap();
        assert!(stp_dev.seen_packets.contains(&PacketType::Stp));
        assert!(stp_dev.stp.is_some());

        let dhcp_dev = scanner
            .devices
            .get(&[0xb8, 0x7b, 0xd4, 0xa8, 0x43, 0xe7])
            .unwrap();
        assert!(dhcp_dev.seen_packets.contains(&PacketType::DhcpRouge));
        assert!(dhcp_dev.dhcp.is_some());
    }
}
