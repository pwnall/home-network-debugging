use anyhow::{Context, Result};
use clap::Parser;
use netgear_cm3000_config::{ModemClient, ModemConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the network configuration JSON file
    #[arg(short, long, default_value = "data/network.json")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

/// The supported subcommands for the Netgear CM3000 utility.
#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// View basic status from the modem's dashboard.
    Status,
    /// View the modem's event logs.
    Logs,
    /// View detailed DOCSIS channel and signal status.
    Docsis,
    /// Refresh the modem's event logs.
    Refresh,
    /// Reboot the cable modem.
    Reboot,
    /// Reset the modem to its factory default settings.
    FactoryReset,
    /// Set the starting frequency (in Hz) for the downstream channel search.
    SetFrequency {
        /// The starting frequency in Hertz.
        #[arg(index = 1)]
        freq_hz: u32,
    },
    /// Set a new administrator password for the modem.
    SetPassword {
        /// The new password.
        #[arg(index = 1)]
        new_pass: String,
    },
    /// Enable or disable Link Aggregation Control Protocol (LACP).
    SetLacp {
        /// Whether LACP should be enabled.
        #[arg(index = 1)]
        enabled: bool,
    },
    /// Enable or disable forced HTTPS for the local management interface.
    SetHttps {
        /// Whether forced HTTPS should be enabled.
        #[arg(index = 1)]
        enabled: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkJson {
    modem: ModemConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Load configuration
    let config_content = fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config file: {:?}", args.config))?;
    let network_data: NetworkJson =
        serde_json::from_str(&config_content).context("Failed to parse network.json")?;

    println!("Connecting to modem at {}...", network_data.modem.ip);

    // 2. Initialize client
    let client = ModemClient::new(&network_data.modem)?;

    // 3. Login
    client
        .login(&network_data.modem)
        .await
        .context("Authentication failed")?;
    println!("Successfully logged in.");

    match args.command {
        Commands::Status => {
            let html = client.fetch_page("/DashBoard.htm").await?;
            let status = ModemClient::parse_dashboard_status(&html)?;
            println!("Modem Status Overview:");
            println!("  Connection:  {}", status.connection_status);
            println!("  IP Address:  {}", status.cm_ip);
            println!("  Firmware:    {}", status.firmware_version);
            println!("  Hardware:    {}", status.hardware_version);
            println!("\n  (Use 'docsis' for detailed signal levels and up-time)");
        }
        Commands::Logs => {
            let html = client.fetch_page("/eventLog.htm").await?;
            let entries = ModemClient::parse_xml_logs(&html)?;
            println!("Event Logs ({} entries):", entries.len());
            println!("{:<25} | {:<15} | Description", "Time", "Priority");
            println!("{:-<25}-+-{:-<15}-+-{:-<30}", "", "", "");
            for entry in entries {
                println!(
                    "{:<25} | {:<15} | {}",
                    entry.time, entry.priority, entry.description
                );
            }
        }
        Commands::Docsis => {
            let html = client.fetch_page("/DocsisStatus.htm").await?;
            let status = ModemClient::parse_docsis_status(&html)?;
            println!("DOCSIS Status:");
            println!("System Time: {}", status.system_time);
            println!("Up Time: {}", status.up_time);
            println!("\nDownstream Bonded Channels:");
            for ch in status.downstream_bonded {
                println!("{:?}", ch);
            }
            println!("\nUpstream Bonded Channels:");
            for ch in status.upstream_bonded {
                println!("{:?}", ch);
            }
        }
        Commands::Refresh => {
            client.refresh_logs().await?;
            println!("Logs refreshed successfully.");
        }
        Commands::Reboot => {
            client.reboot().await?;
            println!("Reboot command sent successfully.");
        }
        Commands::FactoryReset => {
            client.factory_reset().await?;
            println!("Factory reset command sent successfully.");
        }
        Commands::SetFrequency { freq_hz } => {
            client.set_frequency(freq_hz).await?;
            println!("Starting frequency set to {} Hz.", freq_hz);
        }
        Commands::SetPassword { new_pass } => {
            client
                .set_password(&network_data.modem.password, &new_pass)
                .await?;
            println!("Password updated successfully.");
        }
        Commands::SetLacp { enabled } => {
            client.set_lacp(enabled).await?;
            println!(
                "LACP set to {}.",
                if enabled { "enabled" } else { "disabled" }
            );
        }
        Commands::SetHttps { enabled } => {
            client.set_https(enabled).await?;
            println!(
                "HTTPS set to {}.",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }

    Ok(())
}
