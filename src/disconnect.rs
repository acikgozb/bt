use std::{collections::BTreeMap, error, fmt, io, num::ParseIntError};

use tabled::{builder::Builder, settings::Style};

use crate::bluez;

#[derive(Debug)]
pub enum Error {
    DBusClient(bluez::Error),
    Disconnect(bluez::Error),
    Remove(bluez::Error),
    InvalidAlias,
    ConnectedDevices(bluez::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DBusClient(error) => {
                write!(f, "unable to establish a D-Bus connection: {}", error)
            }
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

const LISTING_COLUMNS: [DisconnectColumn; 2] = [DisconnectColumn::Alias, DisconnectColumn::Address];

#[derive(Copy, Clone)]
enum DisconnectColumn {
    Alias,
    Address,
}

impl From<&DisconnectColumn> for String {
    fn from(value: &DisconnectColumn) -> Self {
        let str = match value {
            DisconnectColumn::Alias => "ALIAS",
            DisconnectColumn::Address => "ADDRESS",
        };

        str.to_string()
    }
}

trait Listable {
    fn get_listing_field_by_column(&self, column: &DisconnectColumn) -> String;
}

impl Listable for bluez::Device {
    fn get_listing_field_by_column(&self, column: &DisconnectColumn) -> String {
        let str = match column {
            DisconnectColumn::Alias => self.alias(),
            DisconnectColumn::Address => self.address(),
        };

        str.to_string()
    }
}

pub fn disconnect(
    w: &mut impl io::Write,
    r: &mut impl io::BufRead,
    force: &bool,
    aliases: &Option<Vec<String>>,
) -> Result<(), Error> {
    let bluez = bluez::Client::new().map_err(Error::DBusClient)?;

    let aliases = match aliases.as_ref() {
        Some(aliases) => aliases,
        None => &{ get_aliases_from_user(w, r, &bluez)? },
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
    bluez: &bluez::Client,
) -> Result<Vec<String>, Error> {
    let devices = bluez.connected_devs().map_err(Error::ConnectedDevices)?;
    let mut device_map = BTreeMap::from_iter(devices.iter().enumerate());

    let prompt = [
        &create_device_list(&device_map),
        "\n",
        "Select the device(s) you wish to disconnect: ",
    ]
    .concat();
    w.write_all(prompt.as_bytes())?;
    w.flush()?;

    let mut answer = String::with_capacity(devices.len() * 2);
    r.read_line(&mut answer)?;

    let mut aliases: Vec<String> = Vec::with_capacity(devices.len());
    for idx in answer.split(",") {
        let idx = idx.trim().parse::<u8>()?;
        let device = device_map
            .remove(&(idx as usize))
            .ok_or(Error::InvalidAlias)?;
        aliases.push(device.alias().to_string());
    }

    Ok(aliases)
}

fn create_device_list(device_map: &BTreeMap<usize, &bluez::Device>) -> String {
    let mut table_builder = Builder::new();

    let mut columns = LISTING_COLUMNS.map(|c| String::from(&c)).to_vec();
    columns.insert(0, "IDX".to_string());

    table_builder.push_record(columns);

    for (idx, device) in device_map {
        let mut record = LISTING_COLUMNS
            .map(|c| device.get_listing_field_by_column(&c))
            .to_vec();
        record.insert(0, format!("({})", idx));

        table_builder.push_record(record);
    }

    let mut list = table_builder.build();
    list.with(Style::blank());

    list.to_string()
}
