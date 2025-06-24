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
    bluez: &bluez::Client,
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    force: &bool,
    aliases: &Option<Vec<String>>,
) -> Result<(), Error> {
    let aliases = match aliases.as_ref() {
        Some(aliases) => aliases,
        None => &{
            let devices = bluez.connected_devs().map_err(Error::ConnectedDevices)?;

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
