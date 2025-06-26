use std::{error, fmt, io};

use crate::bluez;

/// Defines error variants that may be returned from a [`status`] call.
///
/// [`status`]: crate::status
#[derive(Debug)]
pub enum Error {
    /// Happens when the power state of the Bluetooth adapter could not be read.
    /// It holds the underlying [`bluez::Error`] error.
    ///
    /// [`bluez::Error`]: crate::bluez::Error
    PowerState(bluez::Error),

    /// Happens when the connected Bluetooth devices could not be read.
    /// It holds the underlying DBus error.
    ConnectedDevices(bluez::Error),

    /// Happens when the result of [`status`] could not be written to the given buffer.
    /// It holds the underlying [`io::Error`].
    ///
    /// [`status`]: crate::status
    /// [`io::Error`]: std::io::Error
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::PowerState(error) => {
                write!(f, "unable to get device power state: {}", error)
            }
            Error::ConnectedDevices(error) => {
                write!(f, "unable to get connected devices: {}", error)
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

/// Provides the Bluetooth adapter status and connected Device-MAC address pairs by using a [`BluezClient`].
///
/// The Bluetooth adapter status and Device-MAC address pairs are written to the provided [`io::Write`].
///
/// The format of the WiFi status depends on [`BluezClient`].
///
/// The format of the Device-MAC address pairs is like below:
///
/// ```txt
/// connected devices:
/// Dev1/Addr1 (batt: Batt1%)
/// Dev1/Addr2 (batt: batt2%)
/// ...
/// DevN/AddrN (batt: battN%)
/// ```
///
/// # Panics
///
/// This function panics when the battery percentage of a connected device is not known.
/// [`status`] assumes that all connected devices have their battery percentages and [`BluezClient`] is able to provide those.
///
/// # Errors
///
/// This function can return all variants of [`StatusError`] based on given conditions. For more details, please see the error documentation.
///
/// [`BluezClient`]: crate::BluezClient
/// [`io::Write`]: std::io::Write
/// [`StatusError`]: crate::StatusError
/// [`status`]: crate::status
///
/// # Examples
///
/// Here is a basic [`status`] call. The output assertion is done to show the format of the success result. The actual output will contain the real connected device aliases and their MAC addresses.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{status, BluezClient};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// let status_result = status(&bluez_client, &mut output);
///
/// assert!(status_result.is_ok());
/// let status_str = String::from_utf8(output.into_inner()).unwrap();
/// assert_eq!(status_output, "bluetooth: enabled\nconnected devices:\nDev1/Addr1\nDev2/Addr2");
///```
///
/// Here is an error case. The example triggers an [`io::Error`] by passing an array as a buffer, instead of a growable buffer.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{status, BluezClient, StatusError};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new([]);
///
/// let status_result = status(&bluez_client, &mut output);
///
/// match status_result {
///     Err(StatusError::Io(err)) => eprintln!("{}", err),
///     _ => unreachable!(),
/// }
///```
pub fn status(bluez: &crate::BluezClient, f: &mut impl io::Write) -> Result<(), Error> {
    let power_state = bluez.power_state().map_err(Error::PowerState)?;
    let connected_devs = bluez.connected_devices().map_err(Error::ConnectedDevices)?;

    let mut buf = [
        "bluetooth: ",
        &power_state.to_string(),
        "\nconnected devices: ",
    ]
    .join("");
    for dev in connected_devs {
        let format = format!(
            "\n{}/{} (batt: %{})",
            dev.alias(),
            dev.address(),
            dev.battery().unwrap()
        );
        buf.push_str(&format)
    }

    f.write_all(buf.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use io::Cursor;

    use super::*;

    #[test]
    fn it_should_write_bluetooth_status() {
        let bluez = crate::BluezClient::new().unwrap();
        let mut out_buf = Cursor::new(vec![]);

        status(&bluez, &mut out_buf).unwrap();

        let connected_device = &bluez.connected_devices().unwrap()[0];
        let expected = format!(
            "bluetooth: enabled\nconnected devices: \n{}/{} (batt: %{})",
            connected_device.alias(),
            connected_device.address(),
            connected_device.battery().unwrap()
        );

        let result = String::from_utf8(out_buf.into_inner()).unwrap();

        assert_eq!(expected, result)
    }

    #[test]
    fn it_should_fail_if_power_state_cannot_be_read() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("power_state".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let result = status(&bluez, &mut out_buf);

        assert!(result.is_err())
    }

    #[test]
    fn it_should_fail_if_connected_devices_cannot_be_read() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("connected_devices".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let result = status(&bluez, &mut out_buf);

        assert!(result.is_err())
    }

    #[test]
    fn it_should_fail_when_result_cannot_be_written_to_buf() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut out_buf = Cursor::new([]);
        out_buf.set_position(1);

        let result = status(&bluez, &mut out_buf);

        assert!(result.is_err())
    }
}
