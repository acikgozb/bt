use std::{error, fmt, io};

use crate::bluez;

#[derive(Debug)]
pub enum Error {
    PowerState(bluez::Error),
    ConnectedDevices(bluez::Error),
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
