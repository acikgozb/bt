use core::fmt;
use std::{error, io};

use clap::{Args, arg};

use crate::{
    BluezError, bluez,
    format::{PrettyFormatter, TableFormattable, TerseFormatter},
};

/// Defines error variants that may be returned from a [`list_devices`] call.
///
/// [`list_devices`]: crate::list_devices
#[derive(Debug)]
pub enum Error {
    /// Happens when the [`BluezClient`] fails during the process.
    /// It holds the underlying [`BluezError`].
    ///
    /// [`BluezError`]: crate::BluezError
    /// [`BluezClient`]: crate::BluezClient
    Bluez(BluezError),

    /// Happens when [`list_devices`] cannot write to the provided [`io::Write`].
    ///
    /// It holds the underlying [`io::Error`].
    ///
    /// [`list_devices`]: crate::list_devices
    /// [`io::Error`]: std::io::Error
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Bluez(error) => {
                write!(f, "list-devices: bluez error: {}", error)
            }
            Error::Io(error) => write!(f, "list-devices: io error: {}", error),
        }
    }
}

impl error::Error for Error {}

impl From<BluezError> for Error {
    fn from(value: BluezError) -> Self {
        Error::Bluez(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Defines the arguments that [`list_devices`] can take.
///
/// [`list_devices`]: crate::list_devices
#[derive(Debug, Args)]
pub struct ListDevicesArgs {
    /// Filter the table output based on given keys.
    #[arg(short, long, value_delimiter = ',')]
    pub columns: Option<Vec<ListDevicesColumn>>,

    /// Filter the terse output based on given keys.
    #[arg(short, long, value_delimiter = ',')]
    pub values: Option<Vec<ListDevicesColumn>>,

    /// Filter output based on device status.
    #[arg(short, long)]
    pub status: Option<DeviceStatus>,
}

/// Defines the columns of a [`list_devices`] output.
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ListDevicesColumn {
    Alias,
    Address,
    Connected,
    Trusted,
    Bonded,
    Paired,
}

/// Defines the available statuses of Bluetooth devices.
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum DeviceStatus {
    Connected,
    Trusted,
    Bonded,
    Paired,
}

impl TableFormattable<ListDevicesColumn> for bluez::BluezDevice {
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
impl TableCellFilter for bluez::BluezDevice {
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

/// Provides a list of known Bluetooth devices on the host by using a [`BluezClient`].
///
/// The list is written to the provided [`io::Write`].
///
/// The format of the list depends on the arguments passed:
///
/// - If `args.columns` are [`Some`], then [`list_devices`] uses the pretty formatting, which is a table.
/// - If `args.values` are [`Some`], then [`list_devices`] uses the terse formatting, which is a listing where each property of the scanned devices are concatenated by the delimiter `/`.
/// - If both `args.columns` and `args.values` are [`Some`], then [`list_devices`] uses the pretty formatting.
/// - If both `args.columns` and `args.values` are [`None`], then [`list_devices`] uses the pretty formatting with the default columns `ALIAS, ADDRESS, CONNECTED, TRUSTED, BONDED, PAIRED`.
///
/// Here is how pretty formatting looks like:
///
/// ```txt
/// ALIAS         ADDRESS             CONNECTED   TRUSTED   BONDED   PAIRED
/// KnownDev1     XX:XX:XX:XX:XX:XX   true        true      false    true
/// KnownDev2     XX:XX:XX:XX:XX:XX   false       true      false    false
/// ```
///
/// Here is how terse formatting looks like:
///
/// ```txt
/// KnownDev1/XX:XX:XX:XX:XX:XX/true/true/false/true
/// KnownDev2/XX:XX:XX:XX:XX:XX/false/true/false/false
/// ```
///
/// The columns can be filtered by the provided [`ListDevicesColumn`] in `args.columns` or `args.values`.
///
/// The devices can be filtered by the provided [`DeviceStatus`] in `args.status`.
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function can return all variants of [`ListDevicesError`] based on given conditions. For more details, please see the error documentation.
///
///
/// # Examples
///
/// Here is a basic [`list_devices`] call that will use pretty formatting with no column or status filtering.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{list_devices, BluezClient, ListDevicesArgs};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// let args = ListDevicesArgs {
///     columns: None,
///     values: None,
///     status: None,
/// };
///
/// let list_dev_result = list_devices(&bluez_client, &mut output, &args);
/// match list_dev_result {
///     Ok(_) => {
///          let pretty_out = String::from_utf8(output.into_inner()).unwrap();
///          println!("{}", pretty_out);
///     },
///     Err(e) => eprintln!("list_devices error: {}", e)
/// }
///```
///
/// Here is an example to showcase how to filter the list. The same filtering can be used for terse formatting by using `args.values` instead.
///
///```no_run
/// use std::io::Cursor;
/// use bt::{list_devices, BluezClient, ListDevicesArgs, ListDevicesColumn};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// // Only ALIAS, CONNECTED, and TRUSTED columns are shown.
/// let args = ListDevicesArgs {
///     columns: Some(vec![ListDevicesColumn::Alias, ListDevicesColumn::Connected, ListDevicesColumn::Trusted]),
///     values: None,
///     status: None,
/// };
///
/// let list_dev_result = list_devices(&bluez_client, &mut output, &args);
/// match list_dev_result {
///     Ok(_) => {
///          let pretty_out = String::from_utf8(output.into_inner()).unwrap();
///          println!("{}", pretty_out);
///     },
///     Err(e) => eprintln!("list_devices error: {}", e)
/// }
/// ```
///
/// Here is an example to showcase how to filter the list by device status'. The same filtering can be used for terse formatting by using `args.values` instead.
///
///```no_run
/// use std::io::Cursor;
/// use bt::{list_devices, BluezClient, ListDevicesArgs, ListDevicesColumn};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// // Only the ALIAS of connected devices are shown.
/// let args = ListDevicesArgs {
///     columns: Some(vec![ListDevicesColumn::Alias]),
///     values: None,
///     status: Some(DeviceStatus::Connected),
/// };
///
/// let list_dev_result = list_devices(&bluez_client, &mut output, &args);
/// match list_dev_result {
///     Ok(_) => {
///          let pretty_out = String::from_utf8(output.into_inner()).unwrap();
///          println!("{}", pretty_out);
///     },
///     Err(e) => eprintln!("list_devices error: {}", e)
/// }
/// ```
///
/// Here is an error case. The example triggers an [`io::Error`] by passing an array as a buffer, instead of a growable buffer.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{list_devices, BluezClient, ListDevicesArgs, ListDevicesError};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new([]);
///
/// let args = ListDevicesArgs {
///     columns: None,
///     values: None,
///     status: None,
/// };
///
/// let list_dev_result = list_devices(&bluez_client, &mut output, &args);
/// match list_dev_result {
///     Err(ListDevicesError::Io(err)) => eprintln!("{}", err),
///     _ => unreachable!(),
/// }
///```
///
/// [`BluezClient`]: crate::BluezClient
/// [`io::Write`]: std::io::Write
/// [`Some`]: std::option::Option::Some
/// [`None`]: std::option::Option::None
/// [`ListDevicesError`]: crate::ListDevicesError
/// [`list_devices`]: crate::list_devices
/// [`ListDevicesArgs`]: crate::ListDevicesArgs
/// [`DeviceStatus`]: crate::DeviceStatus
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

    let devices = bluez.devices()?;
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
