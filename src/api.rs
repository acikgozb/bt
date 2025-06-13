use clap::{Args, Parser, Subcommand, arg, command};

use crate::{list_devices::ListDevicesArgs, scan::ScanArgs};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<BtCommand>,
}

#[derive(Debug, Subcommand)]
pub enum BtCommand {
    /// See Bluetooth status.
    #[clap(visible_alias = "s")]
    Status,

    /// Toggle Bluetooth status.
    #[clap(visible_alias = "t")]
    Toggle,

    #[clap(visible_alias = "ls")]
    ListDevices {
        #[command(flatten)]
        args: ListDevicesArgs,
    },

    /// Scan available devices.
    #[clap(visible_alias = "sc")]
    Scan {
        #[command(flatten)]
        args: ScanArgs,
    },

    /// Connect to an available device.
    #[clap(visible_alias = "c")]
    Connect {
        #[command(flatten)]
        args: ConnectArgs,
    },

    /// Disconnect from an available device.
    #[clap(visible_alias = "d")]
    Disconnect {
        /// Remove the device from the known devices list.
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
}

#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// Set the duration of the scan.
    ///
    /// This option is only available during the interactive mode.
    #[arg(short, long, default_value_t = 5u8, conflicts_with = "alias")]
    pub duration: u8,

    /// Show available devices that contains the name <NAME> in their ALIAS.
    ///
    /// This option is only available during the interactive mode.
    #[arg(short, long, conflicts_with = "alias")]
    pub contains_name: Option<String>,

    /// Connect to ALIAS if it the device is available.
    ///
    /// This option does not initiate a scan, therefore it bypasses the interactive mode completely.
    /// This option expects the full device ALIAS, unlike --contains-name.
    #[arg(short, long)]
    pub alias: Option<String>,
}
