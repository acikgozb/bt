use std::{collections::BTreeMap, error, fmt, io, num::ParseIntError};

use crate::{
    bluez,
    format::{PrettyFormatter, TableFormattable},
};

#[derive(Debug)]
pub enum Error {
    Disconnect(bluez::Error),
    Remove(bluez::Error),
    InvalidAlias,
    ConnectedDevices(bluez::Error),
    NoConnectedDevices,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Disconnect(error) => write!(f, "unable to disconnect: {}", error),
            Error::Remove(error) => write!(f, "unable to remove: {}", error),
            Error::InvalidAlias => write!(f, "the provided alias is invalid"),
            Error::ConnectedDevices(error) => {
                write!(f, "unable to get connected devices: {}", error)
            }
            Error::Io(error) => write!(f, "io error: {}", error),
            Error::NoConnectedDevices => write!(f, "there are no connected devices to disconnect"),
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

impl TableFormattable<DisconnectColumn> for (&usize, &bluez::Device) {
    fn get_cell_value_by_column(&self, column: &DisconnectColumn) -> String {
        match column {
            DisconnectColumn::Idx => self.0.to_string(),
            DisconnectColumn::Alias => self.1.alias().to_string(),
            DisconnectColumn::Address => self.1.address().to_string(),
        }
    }
}

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
            let devices = bluez.connected_devices().map_err(Error::ConnectedDevices)?;

            get_aliases_from_user(w, r, devices)?
        },
    };

    for alias in aliases {
        let alias = alias.trim();

        let disconnect_result = if *force {
            bluez.remove(alias).map_err(Error::Remove)?;
            format!("removed device {} (forced)\n", alias)
        } else {
            bluez.disconnect(alias).map_err(Error::Disconnect)?;
            format!("disconnected from device {}\n", alias)
        };

        w.write_all(disconnect_result.as_bytes())?;
    }

    Ok(())
}

fn get_aliases_from_user(
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    devices: Vec<bluez::Device>,
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
