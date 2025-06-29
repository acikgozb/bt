use std::{collections::BTreeMap, error, fmt, io, num::ParseIntError};

use crate::{
    BluezError, bluez,
    format::{PrettyFormatter, TableFormattable},
};

/// Defines error variants that may be returned from a [`disconnect`] call.
///
/// [`disconnect`]: crate::disconnect
#[derive(Debug)]
pub enum Error {
    /// Happens when the [`BluezClient`] fails during a [`disconnect`] call.
    /// It holds the underlying [`BluezError`].
    ///
    /// [`BluezError`]: crate::BluezError
    /// [`BluezClient`]: crate::BluezClient
    Bluez(BluezError),

    /// Happens when the user selects an invalid alias. This variant may only occur during the interactive mode.
    ///
    /// The selection is invalid when:
    ///
    /// - User enters an index which does not exist on the list.
    /// - User enters something other than the provided indexes.
    InvalidAlias,

    /// Happens when there are no connected devices on the host to disconnect from. This variant may only occur during the interactive mode.
    NoConnectedDevices,

    /// Happens when [`disconnect`] cannot write to the provided [`io::Write`] or cannot read from the provided [`io::BufRead`].
    ///
    /// It holds the underlying [`io::Error`].
    ///
    /// [`disconnect`]: crate::disconnect
    /// [`io::Error`]: std::io::Error
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidAlias => write!(f, "disconnect: the provided alias is invalid"),
            Error::Io(error) => write!(f, "disconnect: io error: {}", error),
            Error::NoConnectedDevices => write!(
                f,
                "disconnect: there are no connected devices to disconnect"
            ),
            Error::Bluez(error) => write!(f, "disconnect: bluez error: {}", error),
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

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidAlias
    }
}

const DEFAULT_LISTING_COLUMNS: [DisconnectColumn; 3] = [
    DisconnectColumn::Idx,
    DisconnectColumn::Alias,
    DisconnectColumn::Address,
];

#[derive(Copy, Clone)]
enum DisconnectColumn {
    Idx,
    Alias,
    Address,
}

impl From<&DisconnectColumn> for String {
    fn from(value: &DisconnectColumn) -> Self {
        let str = match value {
            DisconnectColumn::Idx => "IDX",
            DisconnectColumn::Alias => "ALIAS",
            DisconnectColumn::Address => "ADDRESS",
        };

        str.to_string()
    }
}

impl TableFormattable<DisconnectColumn> for (&usize, &bluez::BluezDevice) {
    fn get_cell_value_by_column(&self, column: &DisconnectColumn) -> String {
        match column {
            DisconnectColumn::Idx => self.0.to_string(),
            DisconnectColumn::Alias => self.1.alias().to_string(),
            DisconnectColumn::Address => self.1.address().to_string(),
        }
    }
}

/// Provides the ability of disconnecting from a connected device by using a [`BluezClient`].
///
/// [`disconnect`] has **interactive** and **non-interactive** modes and they are based on the provided `aliases`.
///
/// # Interactive Mode
///
/// [`disconnect`] runs interactively if `aliases` is [`None`].
///
/// In this mode, [`disconnect`] fetches the connected devices first to find out the device to disconnect from.
///
/// When the devices are fetched, a list is written to the provided [`io::Write`]. The written list is in pretty format (is a table) and has the same columns as what [`connect`] provides except the RSSI column. Like [`connect`], the columns are not customizable.
///
/// The selected IDX of a connected device is read from the provided [`io::BufRead`].
///
/// Here is how the table of connected devices looks like:
///
/// ```txt
/// IDX    ALIAS   ADDRESS          
/// (0)    Dev1    XX:XX:XX:XX:XX:XX
/// (1)    Dev2    XX:XX:XX:XX:XX:XX
/// (2)    Dev3    XX:XX:XX:XX:XX:XX
/// ```
///
/// Once an IDX is selected, [`disconnect`] tries to disconnect from that device by using a [`BluezClient`].
/// Upon disconnecting, [`disconnect`] writes a message to the provided [`io::Write`].
///
/// # Non-Interactive Mode
///
/// [`disconnect`] runs non-interactively if `aliases` is [`Some`].
///
/// In this mode, [`disconnect`] does NOT fetch the connected devices and tries to disconnect from each device through their aliases defined in `aliases`.
///
/// Upon disconnecting, [`disconnect`] writes a messages to the provided [`io::Write`].
///
/// Both modes can be used depending on how convenient defining the `aliases` is.
///
/// In order to see the connected devices, [`list_devices`] or [`status`] can be used.
///
/// # Removing a device
///
/// [`disconnect`] also provides the ability to remove a device completely based on whether `force` is true or not.
///
/// If `force` is `true`, then both interactive and non-interactive mode results in removing the device from the known devices list on the host.
///
/// If `force` is `false`, the both interactive and non-interactive mode results in disconnecting from the device. The device will be kept in the known device list.
///
/// `force` does not change the behavior of interactive and non-interactive mode explained above.
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function can return all variants of [`DisconnectError`] based on given conditions. For more details, please see the error documentation.
///
/// # Examples
///
/// Here is an example for an interactive [`disconnect`]. `force` is `false`, so [`disconnect`] does not remove the device.
///
/// ```no_run
/// use std::io;
/// use bt::{disconnect, BluezClient};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let force = false;
/// let aliases = None;
///
/// // Before returning `disconnect_result`, [`disconnect`] writes the list of connected devices to `output`.
/// // The selection will be read from `input`.
/// let disconnect_result = disconnect(&bluez_client, &mut output, &mut input.lock(), &force, &aliases);
/// match disconnect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("disconnect error: {}", e)
/// }
///```
///
/// In order to remove a connected device, use `force`.
///
///```no_run
/// use std::io;
/// use bt::{disconnect, BluezClient};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let force = true;
/// let aliases = None;
///
/// // Before returning `disconnect_result`, [`disconnect`] writes the list of connected devices to `output`.
/// // The selection will be read from `input`.
/// let disconnect_result = disconnect(&bluez_client, &mut output, &mut input.lock(), &force, &aliases);
/// match disconnect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("connect error: {}", e)
/// }
/// ```
///
/// Here is an example for a non-interactive [`disconnect`]. In this example, `aliases` is set to a vector which holds the ALIAS of the connected device.
///
///```no_run
/// use std::io;
/// use bt::{disconnect, BluezClient};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let force = false;
/// let aliases = Some(vec!["connected_dev".to_string()]);
///
/// // `disconnect` tries to disconnect from the device that has the alias "connected_dev".
/// // It will not show the connected devices.
/// // `output` is only used to provide the success message.
/// let disconnect_result = disconnect(&bluez_client, &mut output, &mut input.lock(), &force, &aliases);
/// match disconnect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("disconnect error: {}", e)
/// }
/// ```
///
/// In order to remove a device in the non-interactive mode, use `force` just as we did in the interactive mode.
///
///```no_run
/// use std::io;
/// use bt::{disconnect, BluezClient};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = io::stdin();
/// let mut output = io::stdout();
///
/// let force = true;
/// let aliases = Some(vec!["connected_dev".to_string()]);
///
/// // `disconnect` tries to remove the device that has the alias "connected_dev".
/// // It will not show the connected devices.
/// // `output` is only used to provide the success message.
/// let disconnect_result = disconnect(&bluez_client, &mut output, &mut input.lock(), &force, &aliases);
/// match disconnect_result {
///     Ok(_) => {
///          // `output` contains the success message.
///          // ...
///     },
///     Err(e) => eprintln!("disconnect error: {}", e)
/// }
/// ```
///
/// Here is an error case. The example triggers an [`io::Error`] by passing an array as a buffer, instead of a growable buffer.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{disconnect, BluezClient, DisconnectError};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut input = Cursor::new([]);
/// let mut output = Cursor::new([]);
///
/// let force = false;
/// let aliases = None;
///
/// let disconnect_result = disconnect(&bluez_client, &mut output, &mut input, &force, &aliases);
/// match disconnect_result {
///     Err(DisconnectError::Io(err)) => eprintln!("{}", err),
///     _ => unreachable!(),
/// }
///```
/// [`BluezClient`]: crate::BluezClient
/// [`io::Write`]: std::io::Write
/// [`io::BufRead`]: std::io::BufRead
/// [`Some`]: std::option::Option::Some
/// [`None`]: std::option::Option::None
/// [`DisconnectError`]: crate::DisconnectError
/// [`disconnect`]: crate::disconnect
/// [`connect`]: crate::connect
/// [`list_devices`]: crate::list_devices
/// [`status`]: crate::status
pub fn disconnect(
    bluez: &crate::BluezClient,
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    force: &bool,
    aliases: &Option<Vec<String>>,
) -> Result<(), Error> {
    let aliases = match aliases.as_ref() {
        Some(aliases) => aliases,
        None => &{
            let devices = bluez.connected_devices()?;

            get_aliases_from_user(w, r, devices)?
        },
    };

    for alias in aliases {
        let alias = alias.trim();

        let disconnect_result = if *force {
            bluez.remove(alias)?;
            format!("removed device {} (forced)\n", alias)
        } else {
            bluez.disconnect(alias)?;
            format!("disconnected from device {}\n", alias)
        };

        w.write_all(disconnect_result.as_bytes())?;
    }

    Ok(())
}

fn get_aliases_from_user(
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    devices: Vec<bluez::BluezDevice>,
) -> Result<Vec<String>, Error> {
    let dev_len = devices.len();
    if dev_len == 0 {
        return Err(Error::NoConnectedDevices);
    }

    let mut device_map = BTreeMap::from_iter(devices.into_iter().enumerate());
    let devices = device_map
        .iter()
        .to_pretty(&DEFAULT_LISTING_COLUMNS)
        .to_string();

    let prompt = [
        &devices,
        "\n",
        "Select the device(s) you wish to disconnect: ",
    ]
    .concat();
    w.write_all(prompt.as_bytes())?;
    w.flush()?;

    let mut answer = String::with_capacity(dev_len * 2);
    r.read_line(&mut answer)?;

    let mut aliases: Vec<String> = Vec::with_capacity(dev_len);
    for idx in answer.split(",") {
        let idx = idx.trim().parse::<u8>()?;
        let device = device_map
            .remove(&(idx as usize))
            .ok_or(Error::InvalidAlias)?;
        aliases.push(device.alias().to_string());
    }

    Ok(aliases)
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Cursor;

    #[test]
    fn it_should_disconnect_if_not_forced() {
        let mut bluez = crate::BluezClient::new().unwrap();
        // NOTE: The Bluez remove is set to err to see that it is not
        // executed by checking res.is_ok().
        bluez.set_erred_method_name("remove".to_string());

        let force = false;

        for aliases in [None, Some(vec!["connected_device".to_string()])] {
            let mut in_buf = match aliases {
                Some(_) => Cursor::new(vec![]),
                None => {
                    let user_device_selection = String::from("0\n");
                    Cursor::new(user_device_selection.as_bytes().to_vec())
                }
            };
            let mut out_buf = Cursor::new(vec![]);

            let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

            assert!(result.is_ok());
            assert!(!out_buf.into_inner().is_empty());
        }
    }

    #[test]
    fn it_should_remove_if_forced() {
        let mut bluez = crate::BluezClient::new().unwrap();
        // NOTE: The Bluez disconnect is set to err to see that it is not
        // executed by checking res.is_ok().
        bluez.set_erred_method_name("disconnect".to_string());

        let force = true;

        for aliases in [None, Some(vec!["connected_device".to_string()])] {
            let mut in_buf = match aliases {
                Some(_) => Cursor::new(vec![]),
                None => {
                    let user_device_selection = String::from("0\n");
                    Cursor::new(user_device_selection.as_bytes().to_vec())
                }
            };
            let mut out_buf = Cursor::new(vec![]);

            let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

            assert!(result.is_ok());
            assert!(!out_buf.into_inner().is_empty());
        }
    }

    #[test]
    fn is_should_show_known_devices_if_alias_is_not_provided() {
        let bluez = crate::BluezClient::new().unwrap();

        let user_device_selection = String::from("0\n");
        let mut in_buf = Cursor::new(user_device_selection.as_bytes().to_vec());
        let mut out_buf = Cursor::new(vec![]);
        let force = false;
        let aliases = None;

        let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

        assert!(result.is_ok());

        let out_buf = out_buf.into_inner();
        assert!(!out_buf.is_empty());

        // NOTE: If known devs are shown, that means the output consists of multiple lines.
        assert!(out_buf.split(|b| b == &b'\n').count() > 1)
    }

    #[test]
    fn it_should_fail_when_it_cannot_get_known_devices() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("connected_devices".to_string());

        let user_device_selection = String::from("0\n");
        let mut in_buf = Cursor::new(user_device_selection.as_bytes().to_vec());
        let mut out_buf = Cursor::new(vec![]);
        let force = false;
        let aliases = None;

        let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

        assert!(result.is_err());

        let out_buf = out_buf.into_inner();
        assert!(out_buf.is_empty());
    }

    #[test]
    fn it_should_fail_when_it_cannot_disconnect() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("disconnect".to_string());

        let force = false;

        for aliases in [None, Some(vec!["connected_device".to_string()])] {
            let mut in_buf = match aliases {
                Some(_) => Cursor::new(vec![]),
                None => {
                    let user_device_selection = String::from("0\n");
                    Cursor::new(user_device_selection.as_bytes().to_vec())
                }
            };
            let mut out_buf = Cursor::new(vec![]);

            let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

            assert!(result.is_err());

            if aliases.is_some() {
                assert!(out_buf.into_inner().is_empty());
            } else {
                assert!(!out_buf.into_inner().is_empty());
            }
        }
    }

    #[test]
    fn it_should_fail_when_it_cannot_remove() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("remove".to_string());

        let force = true;

        for aliases in [None, Some(vec!["connected_device".to_string()])] {
            let mut in_buf = match aliases {
                Some(_) => Cursor::new(vec![]),
                None => {
                    let user_device_selection = String::from("0\n");
                    Cursor::new(user_device_selection.as_bytes().to_vec())
                }
            };
            let mut out_buf = Cursor::new(vec![]);

            let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

            assert!(result.is_err());

            if aliases.is_some() {
                assert!(out_buf.into_inner().is_empty());
            } else {
                assert!(!out_buf.into_inner().is_empty());
            }
        }
    }

    #[test]
    fn it_should_fail_when_result_cannot_be_written_to_buf() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut in_buf = Cursor::new([]);
        let mut out_buf = Cursor::new([]);
        out_buf.set_position(1);
        let force = false;
        let aliases = Some(vec!["connected_device".to_string()]);

        let result = disconnect(&bluez, &mut out_buf, &mut in_buf, &force, &aliases);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty())
    }
}
