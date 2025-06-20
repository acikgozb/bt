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
    let connected_devs = bluez.connected_devs().map_err(Error::ConnectedDevices)?;

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
