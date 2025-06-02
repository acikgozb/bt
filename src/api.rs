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

    #[clap(visible_alias = "ls")]
    ListDevices {
        /// Filter the table output based on given keys.
        #[arg(short, long, value_delimiter = ',')]
        columns: Option<Vec<BtListingKey>>,

        /// Filter the terse output based on given keys.
        #[arg(short, long, value_delimiter = ',')]
        values: Option<Vec<BtListingKey>>,

        /// Filter output based on device status.
        #[arg(short, long)]
        status: Option<BtListingStatusKey>,
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

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum BtListingKey {
    Alias,
    Address,
    Connected,
    Trusted,
    Bonded,
    Paired,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum BtListingStatusKey {
    Connected,
    Trusted,
    Bonded,
    Paired,
}
