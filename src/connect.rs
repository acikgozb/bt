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
    bluez: &crate::BluezClient,
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
    bluez: &crate::BluezClient,
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

#[cfg(test)]
mod tests {

    use super::*;
    use io::Cursor;

    #[test]
    fn it_should_connect_without_scanning_if_alias_is_provided() {
        let mut bluez = crate::BluezClient::new().unwrap();
        // NOTE: The Bluez scan is set to err to see that scan is not
        // executed by checking res.is_ok().
        bluez.set_erred_method_name("start_discovery".to_string());

        let mut in_buf = Cursor::new([]);
        let mut out_buf = Cursor::new(vec![]);

        let connect_args = ConnectArgs {
            duration: Some(0),
            contains_name: None,
            alias: Some("known_dev".to_string()),
        };

        let result = connect(&bluez, &mut out_buf, &mut in_buf, &connect_args);

        assert!(result.is_ok());
        assert!(!out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_run_a_scan_before_connecting_if_alias_is_not_provided() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut out_buf = Cursor::new(vec![]);

        let user_scan_selection = String::from("0\n");
        let mut in_buf = Cursor::new(user_scan_selection.as_bytes().to_vec());

        let connect_args = ConnectArgs {
            duration: Some(0),
            contains_name: None,
            alias: None,
        };

        let result = connect(&bluez, &mut out_buf, &mut in_buf, &connect_args);

        assert!(result.is_ok());
        assert!(!out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_if_interactive_scan_fails() {
        let mut bluez = crate::BluezClient::new().unwrap();

        let user_scan_selection = String::from("0\n");
        let mut in_buf = Cursor::new(user_scan_selection.as_bytes().to_vec());

        let connect_args = ConnectArgs {
            duration: Some(0),
            contains_name: None,
            alias: None,
        };

        for scan_err in ["start_discovery", "scanned_devices", "stop_discovery"] {
            bluez.set_erred_method_name(scan_err.to_string());
            let mut out_buf = Cursor::new(vec![]);

            let result = connect(&bluez, &mut out_buf, &mut in_buf, &connect_args);

            assert!(result.is_err());

            if scan_err != "stop_discovery" {
                assert!(out_buf.into_inner().is_empty());
            } else {
                assert!(!out_buf.into_inner().is_empty());
            }
        }
    }

    #[test]
    fn it_should_fail_if_connect_fails() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("connect".to_string());

        let mut in_buf = Cursor::new([]);
        let mut out_buf = Cursor::new(vec![]);

        let connect_args = ConnectArgs {
            duration: Some(0),
            contains_name: None,
            alias: Some("known_dev".to_string()),
        };

        let result = connect(&bluez, &mut out_buf, &mut in_buf, &connect_args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_when_result_cannot_be_written_to_buf() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut in_buf = Cursor::new([]);
        let mut out_buf = Cursor::new([]);
        out_buf.set_position(1);

        let connect_args = ConnectArgs {
            duration: Some(0),
            contains_name: None,
            alias: Some("known_dev".to_string()),
        };

        let result = connect(&bluez, &mut out_buf, &mut in_buf, &connect_args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty())
    }
}
