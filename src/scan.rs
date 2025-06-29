use core::fmt;
use std::{error, io, thread, time::Duration};

use clap::Args;

use crate::{
    bluez,
    format::{PrettyFormatter, TableFormattable, TerseFormatter},
};

/// Defines error variants that may be returned from a [`scan`] call.
///
/// [`scan`]: crate::scan
#[derive(Debug)]
pub enum Error {
    /// Happens when [`BluezClient`] fails to start the scan.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    /// [`BluezClient`]: crate::BluezClient
    Start(bluez::Error),

    /// Happens when [`BluezClient`] fails to stop the scan.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    /// [`BluezClient`]: crate::BluezClient
    Stop(bluez::Error),

    /// Happens when the scanned devices could not be read.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    DiscoveredDevices(bluez::Error),

    /// Happens when the result of [`scan`] could not be written to the given buffer.
    /// It holds the underlying [`io::Error`].
    ///
    /// [`scan`]: crate::scan
    /// [`io::Error`]: std::io::Error
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Start(error) => write!(f, "unable to start device discovery: {}", error),
            Error::Stop(error) => write!(f, "unable to stop device discovery: {}", error),
            Error::DiscoveredDevices(error) => {
                write!(f, "unable to get discovered devices: {}", error)
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

/// Defines the arguments that [`scan`] can take.
///
/// [`scan`]: crate::scan
#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Set the duration of the scan.
    #[arg(short, long, default_value_t = 5u8)]
    pub duration: u8,

    /// Filter the pretty output based on given columns.
    ///
    /// If no columns are provided, then the full pretty output is shown to the user.
    #[arg(short, long, value_delimiter = ',', num_args = 0.., default_value = None)]
    pub columns: Option<Vec<ScanColumn>>,

    /// Filter the terse output based on given columns.
    ///
    /// If no columns are provided, then the full terse output is shown to the user.
    #[arg(short, long, value_delimiter = ',', num_args = 0.., default_value = None)]
    pub values: Option<Vec<ScanColumn>>,
}

/// Defines the columns that are used to filter the pretty/terse output of [`scan`].
///
/// [`scan`]: crate::scan
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ScanColumn {
    /// Alias shows the alias of the scanned Bluetooth device.
    ///
    /// The actual value of an alias depends on [`BluezClient`].
    ///
    /// [`BluezClient`]: crate::BluezClient
    Alias,

    /// Address shows the MAC address of the scanned Bluetooth device.
    ///
    /// The actual value of a MAC address depends on [`BluezClient`].
    ///
    /// [`BluezClient`]: crate::BluezClient
    Address,

    /// Rssi shows the signal strength of the scanned Bluetooth device.
    ///
    /// The actual value of an Rssi depends on [`BluezClient`].
    ///
    /// [`BluezClient`]: crate::BluezClient
    Rssi,
}

const DEFAULT_LISTING_KEYS: [ScanColumn; 3] =
    [ScanColumn::Alias, ScanColumn::Address, ScanColumn::Rssi];

enum ScanOutput {
    Pretty,
    Terse,
}

impl TableFormattable<ScanColumn> for bluez::Device {
    fn get_cell_value_by_column(&self, column: &ScanColumn) -> String {
        match column {
            ScanColumn::Alias => self.alias().to_string(),
            ScanColumn::Address => self.address().to_string(),
            ScanColumn::Rssi => self.rssi().unwrap_or(0).to_string(),
        }
    }
}

impl From<&ScanColumn> for String {
    fn from(value: &ScanColumn) -> Self {
        let str = match value {
            ScanColumn::Alias => "ALIAS",
            ScanColumn::Address => "ADDRESS",
            ScanColumn::Rssi => "RSSI",
        };

        str.to_string()
    }
}

/// Provides the ability of scanning available devices by using a [`BluezClient`].
///
/// The list of scanned devices are written to the provided [`io::Write`].
///
/// The format of the scanned devices depend on the arguments passed:
///
/// - If `args.columns` are [`Some`], then [`scan`] uses the pretty formatting, which is a table.
/// - If `args.values` are [`Some`], then [`scan`] uses the terse formatting, which is a listing where each property of the scanned devices are concatenated by the delimiter `/`.
/// - If both `args.columns` and `args.values` are [`Some`], then [`scan`] uses the pretty formatting.
/// - If both `args.columns` and `args.values` are [`None`], then [`scan`] uses the pretty formatting with the default columns `ALIAS, ADDRESS, RSSI`.
///
/// Here is how pretty formatting looks like:
///
/// ```txt
/// ALIAS   ADDRESS             RSSI
/// Dev1    XX:XX:XX:XX:XX:XX   -68
/// Dev2    XX:XX:XX:XX:XX:XX   -94
/// Dev3    XX:XX:XX:XX:XX:XX   -93
/// ```
///
/// Here is how terse formatting looks like:
///
/// ```txt
/// Dev1/XX:XX:XX:XX:XX:XX/-92
/// Dev2/XX:XX:XX:XX:XX:XX/-97
/// Dev3/XX:XX:XX:XX:XX:XX/-94
/// ```
///
/// The scan duration can be adjusted by providing `args.duration` of [`ScanArgs`].
/// Setting `args.duration` to 0 is not recommended since a certain amount of time needs to be passed to discover available devices.
///
/// [`scan`] is a blocking call. It blocks the current thread by `args.duration` seconds.
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function can return all variants of [`ScanError`] based on given conditions. For more details, please see the error documentation.
///
/// [`BluezClient`]: crate::BluezClient
/// [`io::Write`]: std::io::Write
/// [`Some`]: std::option::Option::Some
/// [`ScanError`]: crate::ScanError
/// [`scan`]: crate::scan
/// [`ScanArgs`]: crate::ScanArgs
///
/// # Examples
///
/// Here is a basic [`scan`] call that will use pretty formatting.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{scan, BluezClient, ScanArgs};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// let args = ScanArgs {
///     duration: 5,
///     columns: None,
///     values: None,
/// };
///
/// let scan_result = scan(&bluez_client, &mut output, &args);
/// match scan_result {
///     Ok(_) => {
///          let pretty_out = String::from_utf8(output.into_inner()).unwrap();
///          println!("{}", pretty_out);
///     },
///     Err(e) => eprintln!("scan error: {}", e)
/// }
///```
///
/// Here is an example to showcase how to filter the scan output. The same filtering can be used for terse formatting by using `args.values` instead.
///
///```no_run
/// use std::io::Cursor;
/// use bt::{scan, BluezClient, ScanArgs, ScanColumn};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// # The address column is stripped out from the output.
/// let args = ScanArgs {
///     duration: 5,
///     columns: Some(vec![ScanColumn::Alias, ScanColumn::Rssi]),
///     values: None,
/// };
///
/// let scan_result = scan(&bluez_client, &mut output, &args);
/// match scan_result {
///     Ok(_) => {
///          let pretty_out = String::from_utf8(output.into_inner()).unwrap();
///          println!("{}", pretty_out);
///     },
///     Err(e) => eprintln!("scan error: {}", e)
/// }
/// ```
///
/// Here is an error case. The example triggers an [`io::Error`] by passing an array as a buffer, instead of a growable buffer.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{scan, BluezClient, ScanArgs, ScanError};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new([]);
///
/// let args = ScanArgs {
///     duration: 5,
///     columns: None,
///     values: None,
/// };
///
/// let scan_result = scan(&bluez_client, &mut output, &args);
///
/// match scan_result {
///     Err(ScanError::Io(err)) => eprintln!("{}", err),
///     _ => unreachable!(),
/// }
///```
pub fn scan(
    bluez: &crate::BluezClient,
    f: &mut impl io::Write,
    args: &ScanArgs,
) -> Result<(), Error> {
    let (out_format, listing_keys) = match (&args.columns, &args.values) {
        (None, None) => (ScanOutput::Pretty, &DEFAULT_LISTING_KEYS.to_vec()),
        (None, Some(v)) => (
            ScanOutput::Terse,
            if v.is_empty() {
                &DEFAULT_LISTING_KEYS.to_vec()
            } else {
                v
            },
        ),
        (Some(c), _) => (
            ScanOutput::Pretty,
            if c.is_empty() {
                &DEFAULT_LISTING_KEYS.to_vec()
            } else {
                c
            },
        ),
    };

    bluez.start_discovery().map_err(Error::Start)?;
    thread::sleep(Duration::from_secs(u64::from(args.duration)));

    let scanned_devices = bluez.scanned_devices().map_err(Error::DiscoveredDevices)?;

    let devices_iter = scanned_devices.into_iter();
    let out_buf = match out_format {
        ScanOutput::Pretty => devices_iter.to_pretty(listing_keys).to_string(),
        ScanOutput::Terse => devices_iter.to_terse(listing_keys).to_string(),
    };

    f.write_all(out_buf.as_bytes())?;

    bluez.stop_discovery().map_err(Error::Stop)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Cursor;

    #[test]
    fn it_should_write_scanned_devices() {
        let bluez = crate::BluezClient::new().unwrap();
        let mut out_buf = Cursor::new(vec![]);

        let scan_args = ScanArgs {
            duration: 0,
            columns: None,
            values: None,
        };

        let result = scan(&bluez, &mut out_buf, &scan_args);

        assert!(result.is_ok());
        assert!(!out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_when_scan_is_not_started() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("start_discovery".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let scan_args = ScanArgs {
            duration: 0,
            columns: None,
            values: None,
        };

        let result = scan(&bluez, &mut out_buf, &scan_args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_when_scanned_devices_are_not_read() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("scanned_devices".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let scan_args = ScanArgs {
            duration: 0,
            columns: None,
            values: None,
        };

        let result = scan(&bluez, &mut out_buf, &scan_args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_when_scan_is_not_stopped() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("stop_discovery".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let scan_args = ScanArgs {
            duration: 0,
            columns: None,
            values: None,
        };

        let result = scan(&bluez, &mut out_buf, &scan_args);

        assert!(result.is_err());
        assert!(!out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_when_result_cannot_be_written_to_buf() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut out_buf = Cursor::new([]);
        out_buf.set_position(1);

        let scan_args = ScanArgs {
            duration: 0,
            columns: None,
            values: None,
        };

        let result = scan(&bluez, &mut out_buf, &scan_args);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty())
    }
}
