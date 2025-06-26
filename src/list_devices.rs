use core::fmt;
use std::{error, io};

use clap::{Args, arg};

use crate::{
    bluez,
    format::{PrettyFormatter, TableFormattable, TerseFormatter},
};

#[derive(Debug)]
pub enum Error {
    KnownDevices(bluez::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::KnownDevices(error) => {
                write!(f, "unable to get known bluetooth devices: {}", error)
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

#[derive(Debug, Args)]
pub struct ListDevicesArgs {
    /// Filter the table output based on given keys.
    #[arg(short, long, value_delimiter = ',')]
    columns: Option<Vec<ListDevicesColumn>>,

    /// Filter the terse output based on given keys.
    #[arg(short, long, value_delimiter = ',')]
    values: Option<Vec<ListDevicesColumn>>,

    /// Filter output based on device status.
    #[arg(short, long)]
    status: Option<DeviceStatus>,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ListDevicesColumn {
    Alias,
    Address,
    Connected,
    Trusted,
    Bonded,
    Paired,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum DeviceStatus {
    Connected,
    Trusted,
    Bonded,
    Paired,
}

impl TableFormattable<ListDevicesColumn> for bluez::Device {
    fn get_cell_value_by_column(&self, column: &ListDevicesColumn) -> String {
        match column {
            ListDevicesColumn::Alias => self.alias().to_string(),
            ListDevicesColumn::Address => self.address().to_string(),
            ListDevicesColumn::Connected => self.connected().to_string(),
            ListDevicesColumn::Trusted => self.trusted().to_string(),
            ListDevicesColumn::Bonded => self.bonded().to_string(),
            ListDevicesColumn::Paired => self.paired().to_string(),
        }
    }
}

impl From<&ListDevicesColumn> for String {
    fn from(value: &ListDevicesColumn) -> Self {
        let str = match value {
            ListDevicesColumn::Alias => "ALIAS",
            ListDevicesColumn::Address => "ADDRESS",
            ListDevicesColumn::Connected => "CONNECTED",
            ListDevicesColumn::Trusted => "TRUSTED",
            ListDevicesColumn::Bonded => "BONDED",
            ListDevicesColumn::Paired => "PAIRED",
        };

        str.to_string()
    }
}

pub trait TableCellFilter {
    fn filter_cell_value_by_status(&self, key: &DeviceStatus) -> bool;
}
impl TableCellFilter for bluez::Device {
    fn filter_cell_value_by_status(&self, key: &DeviceStatus) -> bool {
        match key {
            DeviceStatus::Connected => self.connected(),
            DeviceStatus::Trusted => self.trusted(),
            DeviceStatus::Bonded => self.bonded(),
            DeviceStatus::Paired => self.paired(),
        }
    }
}

const DEFAULT_LISTING_COLUMNS: [ListDevicesColumn; 6] = [
    ListDevicesColumn::Alias,
    ListDevicesColumn::Address,
    ListDevicesColumn::Connected,
    ListDevicesColumn::Trusted,
    ListDevicesColumn::Bonded,
    ListDevicesColumn::Paired,
];

enum ListDevicesOutput {
    Pretty,
    Terse,
}

pub fn list_devices(
    bluez: &crate::BluezClient,
    f: &mut impl io::Write,
    args: &ListDevicesArgs,
) -> Result<(), Error> {
    let (out_format, user_listing_keys) = match (&args.columns, &args.values) {
        (None, None) => (ListDevicesOutput::Pretty, None),
        (None, values) => (ListDevicesOutput::Terse, values.as_ref()),
        (columns, _) => (ListDevicesOutput::Pretty, columns.as_ref()),
    };

    let listing_keys = match user_listing_keys {
        Some(keys) => keys,
        None => &DEFAULT_LISTING_COLUMNS.to_vec(),
    };

    let devices = bluez.devices().map_err(Error::KnownDevices)?;
    let devices = devices.into_iter().filter(|d| match &args.status {
        Some(s) => d.filter_cell_value_by_status(s),
        None => true,
    });

    let out_buf = match out_format {
        ListDevicesOutput::Pretty => devices.to_pretty(listing_keys).to_string(),
        ListDevicesOutput::Terse => devices.to_terse(listing_keys).to_string(),
    };

    f.write_all(out_buf.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::list_devices;

    use super::*;
    use io::Cursor;

    #[test]
    fn it_should_show_devices() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut out_buf = Cursor::new(vec![]);

        let args = ListDevicesArgs {
            columns: None,
            values: None,
            status: None,
        };

        let result = list_devices(&bluez, &mut out_buf, &args);

        assert!(result.is_ok());
        assert!(!out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_if_it_cannot_get_known_devices() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("devices".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let args = ListDevicesArgs {
            columns: None,
            values: None,
            status: None,
        };

        let result = list_devices(&bluez, &mut out_buf, &args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_filter_devices_based_on_status() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut unfiltered_out_buf = Cursor::new(vec![]);
        let mut filtered_out_buf = Cursor::new(vec![]);

        let mut args = ListDevicesArgs {
            columns: None,
            values: None,
            status: None,
        };

        let result = list_devices(&bluez, &mut unfiltered_out_buf, &args);
        assert!(result.is_ok());
        let unfiltered_len = unfiltered_out_buf.into_inner().len();

        // NOTE: There are no bonded devices returning from BluezTestClient.
        args.status = Some(DeviceStatus::Bonded);

        let result = list_devices(&bluez, &mut filtered_out_buf, &args);
        assert!(result.is_ok());
        let filtered_len = filtered_out_buf.into_inner().len();

        assert!(unfiltered_len > filtered_len);
    }

    #[test]
    fn it_should_fail_when_result_cannot_be_written_to_buf() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut out_buf = Cursor::new([]);

        let args = ListDevicesArgs {
            columns: None,
            values: None,
            status: None,
        };

        let result = list_devices(&bluez, &mut out_buf, &args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty())
    }
}
