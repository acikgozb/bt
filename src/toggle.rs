use std::{error, fmt, io};

use crate::BluezError;

/// Defines error variants that may be returned from a [`toggle`] call.
///
/// [`toggle`]: crate::toggle
#[derive(Debug)]
pub enum Error {
    /// Happens when the [`BluezClient`] fails during the process.
    /// It holds the underlying [`BluezError`].
    ///
    /// [`BluezError`]: crate::BluezError
    /// [`BluezClient`]: crate::BluezClient
    Bluez(BluezError),

    /// Happens when the result of [`toggle`] could not be written to the given buffer.
    /// It holds the underlying [`io::Error`].
    ///
    /// [`toggle`]: crate::toggle
    /// [`io::Error`]: std::io::Error
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::Bluez(error) => {
                write!(f, "toggle: bluez error: {}", error)
            }
            Error::Io(error) => write!(f, "toggle: io error: {}", error),
        }
    }
}

impl error::Error for Error {}

impl From<BluezError> for Error {
    fn from(value: BluezError) -> Self {
        Self::Bluez(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Provides the ability of toggling the Bluetooth adapter status by using a [`BluezClient`].
///
/// The updated Bluetooth adapter status is written to the provided [`io::Write`].
///
/// The format of the Bluetooth status depends on [`BluezClient`].
///
/// # Panics
///
/// This function does not panic.
///
/// # Errors
///
/// This function can return all variants of [`ToggleError`] based on given conditions. For more details, please see the error documentation.
///
/// [`BluezClient`]: crate::BluezClient
/// [`io::Write`]: std::io::Write
/// [`ToggleError`]: crate::ToggleError
/// [`toggle`]: crate::toggle
///
/// # Examples
///
/// Here is a basic [`toggle`] call. The output assertion is done to show the format of the success result. The actual output will contain the exact same state of your Bluetooth adapter after the toggle.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{toggle, BluezClient};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new(vec![]);
///
/// let toggle_result = toggle(&bluez_client, &mut output);
///
/// assert!(toggle_result.is_ok());
/// let toggle_str = String::from_utf8(output.into_inner()).unwrap();
/// assert_eq!(toggle_str, "bluetooth: disabled");
///```
///
/// Here is an error case. The example triggers an [`io::Error`] by passing an array as a buffer, instead of a growable buffer.
///
/// ```no_run
/// use std::io::Cursor;
/// use bt::{toggle, BluezClient, ToggleError};
///
/// let bluez_client = BluezClient::new().unwrap();
/// let mut output = Cursor::new([]);
///
/// let toggle_result = toggle(&bluez_client, &mut output);
///
/// match toggle_result {
///     Err(ToggleError::Io(err)) => eprintln!("{}", err),
///     _ => unreachable!(),
/// }
///```
pub fn toggle(bluez: &crate::BluezClient, f: &mut impl io::Write) -> Result<(), Error> {
    let toggled_power_state = bluez.toggle_power_state()?;

    let buf = format!("bluetooth: {}", toggled_power_state);
    f.write_all(buf.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::Cursor;

    #[test]
    fn it_should_write_toggled_power_state() {
        let bluez = crate::BluezClient::new().unwrap();
        let mut out_buf = Cursor::new(vec![]);

        let result = toggle(&bluez, &mut out_buf);

        assert!(result.is_ok());
        assert!(!out_buf.into_inner().is_empty());
    }

    #[test]
    fn it_should_fail_when_cannot_toggle() {
        let mut bluez = crate::BluezClient::new().unwrap();
        bluez.set_erred_method_name("toggle_power_state".to_string());

        let mut out_buf = Cursor::new(vec![]);

        let result = toggle(&bluez, &mut out_buf);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty())
    }

    #[test]
    fn it_should_fail_when_result_cannot_be_written_to_buf() {
        let bluez = crate::BluezClient::new().unwrap();

        let mut out_buf = Cursor::new([]);
        out_buf.set_position(1);

        let result = toggle(&bluez, &mut out_buf);

        assert!(result.is_err());
        assert!(out_buf.into_inner().is_empty())
    }
}
