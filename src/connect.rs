use std::{collections::BTreeMap, error, fmt, io, num::ParseIntError, thread, time::Duration};

use clap::Args;

use crate::{
    bluez::{self},
    format::{PrettyFormatter, TableFormattable},
};

/// Defines error variants that may be returned from a [`connect`] call.
///
/// [`connect`]: crate::connect
#[derive(Debug)]
pub enum Error {
    /// Happens when [`BluezClient`] fails to start the scan. This variant may only occur during the interactive mode.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    /// [`BluezClient`]: crate::BluezClient
    StartDiscovery(bluez::Error),

    /// Happens when the scanned devices could not be read. This variant may only occur during the interactive mode.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    DiscoveredDevices(bluez::Error),

    /// Happens when [`BluezClient`] fails to stop the scan. This variant may only occur during the interactive mode.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    /// [`BluezClient`]: crate::BluezClient
    StopDiscovery(bluez::Error),

    /// Happens when the connection attempt fails.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    Connect(bluez::Error),

    /// Happens when the user selects an invalid alias. This variant may only occur during the interactive mode.
    ///
    /// The selection is invalid when:
    ///
    /// - User enters an index which does not exist on the list.
    /// - User enters something other than the provided indexes.
    InvalidAlias,

    /// Happens when the result of [`connect`] could not be written to the given buffer.
    /// It holds the underlying [`io::Error`].
    ///
    /// [`connect`]: crate::connect
    /// [`io::Error`]: std::io::Error
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

/// Defines the arguments that [`connect`] can take.
///
/// [`connect`]: crate::connect
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

/// Provides the ability of establishing a connection to an available device by using a [`BluezClient`].
///
/// [`connect`] has **interactive** and **non-interactive** modes and they are based on the provided [`ConnectArgs`].
///
/// # Interactive Mode
///
/// [`connect`] runs interactively if `args.alias` is [`None`].
///
/// In this mode, [`connect`] initiates a Bluetooth scan first to find out the available devices to connect.
///
/// The scanned devices can be filtered by their ALIAS by providing `args.contains_name`. This argument is expected to be a simple substring of the target ALIAS. It is NOT a regex pattern. Please see the examples for its usage.
///
/// The interactive scan is blocking, similar to [`scan`]. It blocks the current thread by 5 seconds and this duration can be adjusted by setting `args.duration`. Setting `args.duration` to 0 is not recommended since a certain amount of time needs to be passed to discover available devices.
///
/// When the scan is completed, the scanned devices are written to the provided [`io::Write`]. The written list is in pretty format (is a table) and has the same columns as what [`scan`] provides with the addition of IDX column. Unlike [`scan`], the columns or the formatting are not customizable.
///
/// The selected IDX of a scanned device is read from the provided [`io::BufRead`].
///
/// Here is how the table of scanned devices looks like:
///
/// ```txt
/// IDX    ALIAS   ADDRESS             RSSI
/// (0)    Dev1    XX:XX:XX:XX:XX:XX   -68
/// (1)    Dev2    XX:XX:XX:XX:XX:XX   -94
/// (2)    Dev3    XX:XX:XX:XX:XX:XX   -93
/// ```
///
/// Once an IDX is selected, [`connect`] tries to establish a connection by using a [`BluezClient`].
/// Upon establishing a connection, [`connect`] writes a message to the provided [`io::Write`].
///
/// # Non-Interactive Mode
///
/// [`connect`] runs non-interactively if `args.alias` is [`Some`].
///
/// In this mode, [`connect`] does NOT initiate a scan and tries to establish a connection to the device by the provided `args.alias`.
///
/// Upon establishing a connection, [`connect`] writes a messages to the provided [`io::Write`].
///
/// This mode should be preferred to the interactive mode if the device is known by the host.
///
/// In order to see whether the device is known or not, [`list_devices`] can be used.
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function can return all variants of [`ConnectError`] based on given conditions. For more details, please see the error documentation.
///
/// # Examples
///
/// Here is an example for an interactive [`connect`]. In this example, the interactive scan is not filtered and its duration is set to `5` seconds (default).
///
/// ```no_run
/// use std::io;
/// use bt::{connect, BluezClient, ConnectArgs};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let args = ConnectArgs {
///     duration: None,
///     contains_name: None,
///     alias: None,
/// };
///
/// // Before returning `connect_result`, [`connect`] writes the list of scanned devices to `output`.
/// // The selection will be read from `input`.
/// let connect_result = connect(&bluez_client, &mut output, &mut input, &args);
/// match connect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("connect error: {}", e)
/// }
///```
///
/// Here is another example for an interactive [`connect`]. In this example, the interactive scan is filtered by `args.contains_name` to only see the available devices which contains the name "dev". The duration is set to [`None`] to use the default (`5` seconds).
///
///```no_run
/// use std::io;
/// use bt::{connect, BluezClient, ConnectArgs};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let args = ConnectArgs {
///     duration: None,
///     contains_name: Some("dev".to_string()),
///     alias: None,
/// };
///
/// // Before returning `connect_result`, [`connect`] writes the list of scanned devices to `output`.
/// // The selection will be read from `input`.
/// let connect_result = connect(&bluez_client, &mut output, &mut input, &args);
/// match connect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("connect error: {}", e)
/// }
/// ```
///
/// Here is an example for a non-interactive [`connect`]. In this example, `args.alias` is set to the ALIAS of the known device that we want to connect directly.
///
///```no_run
/// use std::io;
/// use bt::{connect, BluezClient, ConnectArgs};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let args = ConnectArgs {
///     duration: None,
///     contains_name: None,
///     alias: Some("known_dev".to_string()),
/// };
///
/// // `connect` tries to connect to a device that has the alias "known_dev".
/// // There is no scanning here.
/// // `output` is only used to provide the success message.
/// let connect_result = connect(&bluez_client, &mut output, &mut input, &args);
/// match connect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("connect error: {}", e)
/// }
/// ```
///
/// Here is an error case. The example triggers an [`io::Error`] by passing an array as a buffer, instead of a growable buffer.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{connect, BluezClient, ConnectArgs, ConnectError};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = Cursor::new([]);
/// let mut output = Cursor::new([]);
///
/// let args = ConnectArgs {
///     duration: None,
///     contains_name: None,
///     alias: Some("known_dev".to_string()),
/// };
///
/// let connect_result = connect(&bluez_client, &mut output, &mut input, &args);
/// match connect_result {
///     Err(ConnectError::Io(err)) => eprintln!("{}", err),
///     _ => unreachable!(),
/// }
///```
/// [`BluezClient`]: crate::BluezClient
/// [`io::Write`]: std::io::Write
/// [`io::BufRead`]: std::io::BufRead
/// [`Some`]: std::option::Option::Some
/// [`ConnectError`]: crate::ScanError
/// [`ConnectArgs`]: crate::ConnectArgs
/// [`connect`]: crate::connect
/// [`scan`]: crate::scan
/// [`list_devices`]: crate::list_devices
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
