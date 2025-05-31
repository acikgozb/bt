use clap::{Args, Parser, Subcommand, arg, command};

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
        args: ScanArgs,
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
pub struct ScanArgs {
    /// Set the duration of the scan.
    #[arg(short, long, default_value_t = 5u8)]
    duration: u8,
}
