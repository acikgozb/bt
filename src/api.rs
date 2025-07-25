//! Defines the members which allow the callers to implement a CLI application through this crate.

use clap::{Parser, Subcommand, arg, command};

use crate::{connect::ConnectArgs, list_devices::ListDevicesArgs, scan::ScanArgs};

/// The main CLI struct that holds all subcommands.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<BtCommand>,
}

/// Defines each individual functionality of this crate as a subcommand of a CLI application.
///
/// For more details, please refer to each module that corresponds to each subcommand:
///
/// - `BtCommand::Status`: [`status`]
/// - `BtCommand::Toggle`: [`toggle`]
/// - `BtCommand::list_devices`: [`list_devices`]
/// - `BtCommand::scan`: [`scan`]
/// - `BtCommand::connect`: [`connect`]
/// - `BtCommand::disconnect`: [`disconnect`]
///
/// [`status`]: crate::status
/// [`toggle`]: crate::toggle
/// [`list_devices`]: crate::list_devices
/// [`scan`]: crate::scan
/// [`connect`]: crate::connect
/// [`disconnect`]: crate::disconnect
#[derive(Debug, Subcommand)]
pub enum BtCommand {
    /// See Bluetooth status.
    #[clap(visible_alias = "s")]
    Status,

    /// Toggle Bluetooth status.
    #[clap(visible_alias = "t")]
    Toggle,

    #[clap(visible_alias = "ls")]
    /// See known Bluetooth devices on the host.
    ListDevices {
        #[command(flatten)]
        args: ListDevicesArgs,
    },

    /// Scan available Bluetooth devices.
    #[clap(visible_alias = "sc")]
    Scan {
        #[command(flatten)]
        args: ScanArgs,
    },

    /// Connect to an available Bluetooth device.
    #[clap(visible_alias = "c")]
    Connect {
        #[command(flatten)]
        args: ConnectArgs,
    },

    /// Disconnect from the connected device(s).
    #[clap(visible_alias = "d")]
    Disconnect {
        /// Remove the device(s) from the known devices list.
        #[arg(short, long, default_value_t = false)]
        force: bool,

        /// Disconnect by specifying the full ALIAS of device(s).
        ///
        /// If this argument is not provided, then disconnect first shows the list of connected devices to let users choose. (interactive mode)
        ///
        /// If this argument is provided, then disconnect does not show the list. (non-interactive mode)
        #[arg(value_name = "ALIAS", value_delimiter = ',', num_args = 0.., default_value = None)]
        aliases: Option<Vec<String>>,
    },
}
