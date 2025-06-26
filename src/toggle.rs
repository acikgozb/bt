use std::{error, fmt, io};

use crate::bluez;

#[derive(Debug)]
pub enum Error {
    PowerState(bluez::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::PowerState(error) => {
                write!(f, "unable to toggle device power state: {}", error)
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

pub fn toggle(bluez: &crate::BluezClient, f: &mut impl io::Write) -> Result<(), Error> {
    let toggled_power_state = bluez.toggle_power_state().map_err(Error::PowerState)?;

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
