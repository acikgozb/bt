use std::{error, fmt, io};

use crate::bluez;

#[derive(Debug)]
pub enum Error {
    PowerState(bluez::Error),
    DBusClient(bluez::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::PowerState(error) => {
                write!(f, "unable to toggle device power state: {}", error)
            }
            Error::DBusClient(error) => {
                write!(f, "unable to establish a D-Bus connection: {}", error)
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

pub fn toggle(f: &mut impl io::Write) -> Result<(), Error> {
    let bluez = bluez::Client::new().map_err(Error::DBusClient)?;
    let toggled_power_state = bluez.toggle_power_state().map_err(Error::PowerState)?;

    let buf = format!("bluetooth: {}", toggled_power_state);
    f.write_all(buf.as_bytes())?;

    Ok(())
}
