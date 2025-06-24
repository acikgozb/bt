use std::{collections::BTreeMap, error, fmt, io, num::ParseIntError, thread, time::Duration};

use clap::Args;

use crate::{
    bluez::{self},
    format::{PrettyFormatter, TableFormattable},
};

#[derive(Debug)]
pub enum Error {
    StartDiscovery(bluez::Error),
    DiscoveredDevices(bluez::Error),
    StopDiscovery(bluez::Error),
    Connect(bluez::Error),
    InvalidAlias,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::StartDiscovery(error) => {
                write!(f, "unable to start device discovery: {}", error)
            }
            Error::DiscoveredDevices(error) => {
                write!(f, "unable to get discovered devices: {}", error)
            }
            Error::StopDiscovery(error) => write!(f, "unable to stop device discovery: {}", error),
            Error::Connect(error) => {
                write!(f, "unable to connect to device: {}", error)
            }
            Error::InvalidAlias => {
                write!(f, "the selected alias is not valid")
            }
            Error::Io(error) => write!(f, "io error: {}", error),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidAlias
    }
}

#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// Set the duration of the interactive scan.
    ///
    /// This option has no effect if the device ALIAS is provided.
    #[arg(short, long)]
    pub duration: Option<u8>,

    /// Only show devices that contains the name <CONTAINS_NAME> during the interactive scan.
    ///
    /// This option has no effect if the device ALIAS is provided.
    #[arg(short, long)]
    pub contains_name: Option<String>,

    /// Connect to a known device via its full device ALIAS.
    ///
    /// The ALIAS provided must be the full device ALIAS, unlike --contains-name.
    ///
    /// If this argument is not provided, then connect first initiates a scan to let users choose a device ALIAS. (interactive mode)
    ///
    /// If this argument is provided, then connect does not initiate a scan and attempts to connect to a known device via ALIAS. (non-interactive mode)
    pub alias: Option<String>,
}

#[derive(Clone, Copy)]
enum ConnectColumn {
    Idx,
    Alias,
    Address,
    Rssi,
}

impl From<&ConnectColumn> for String {
    fn from(value: &ConnectColumn) -> Self {
        let str = match value {
            ConnectColumn::Idx => "IDX",
            ConnectColumn::Alias => "ALIAS",
            ConnectColumn::Address => "ADDRESS",
            ConnectColumn::Rssi => "RSSI",
        };

        str.to_string()
    }
}

impl TableFormattable<ConnectColumn> for (&usize, &bluez::Device) {
    fn get_cell_value_by_column(&self, column: &ConnectColumn) -> String {
        match column {
            ConnectColumn::Idx => format!("({})", self.0),
            ConnectColumn::Alias => self.1.alias().to_string(),
            ConnectColumn::Address => self.1.address().to_string(),
            ConnectColumn::Rssi => match self.1.rssi() {
                Some(rssi) => rssi.to_string(),
                None => "-".to_string(),
            },
        }
    }
}

const DEFAULT_LISTING_COLUMNS: [ConnectColumn; 4] = [
    ConnectColumn::Idx,
    ConnectColumn::Alias,
    ConnectColumn::Address,
    ConnectColumn::Rssi,
];

pub fn connect(
    bluez: &bluez::Client,
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    args: &ConnectArgs,
) -> Result<(), Error> {
    let (alias, did_scan) = match &args.alias {
        Some(a) => (a, false),
        None => (
            &{
                let devices = scan_devices(bluez, &args.duration, &args.contains_name)?;

                read_device_alias(w, r, devices)?
            },
            true,
        ),
    };

    bluez.connect(alias).map_err(Error::Connect)?;

    let out_buf = format!("connected to device: {}", alias);
    w.write_all(out_buf.as_bytes())?;

    if did_scan {
        bluez.stop_discovery().map_err(Error::StopDiscovery)?;
    }

    Ok(())
}

fn scan_devices(
    bluez: &bluez::Client,
    duration: &Option<u8>,
    contains_name: &Option<String>,
) -> Result<Vec<bluez::Device>, Error> {
    bluez.start_discovery().map_err(Error::StartDiscovery)?;

    let scan_duration = u64::from(duration.unwrap_or(5));
    thread::sleep(Duration::from_secs(scan_duration));

    let scan_result = bluez.scanned_devices().map_err(Error::DiscoveredDevices)?;
    Ok(match contains_name {
        Some(name) => scan_result
            .into_iter()
            .filter(|d| d.alias().contains(name))
            .collect(),
        None => scan_result,
    })
}

fn read_device_alias(
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    devices: Vec<bluez::Device>,
) -> Result<String, Error> {
    let mut device_map: BTreeMap<usize, bluez::Device> =
        BTreeMap::from_iter(devices.into_iter().enumerate());

    let devices = device_map
        .iter()
        .to_pretty(&DEFAULT_LISTING_COLUMNS)
        .to_string();

    let prompt = [&devices, "\n", "Select the device you wish to connect: "].concat();
    w.write_all(prompt.as_bytes())?;
    w.flush()?;

    let mut read_buf = String::with_capacity(1);
    r.read_line(&mut read_buf)?;

    let selected_idx = read_buf.trim().parse::<u8>()?;
    let selected_device = device_map
        .remove(&(selected_idx as usize))
        .ok_or(Error::InvalidAlias)?;

    Ok(selected_device.alias().to_string())
}
