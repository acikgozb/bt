use core::fmt;
use std::{error, io};

use clap::{Args, arg};
use tabled::{builder::Builder, settings::Style};

use crate::bluez;

#[derive(Debug)]
pub enum Error {
    KnownDevices(bluez::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::KnownDevices(error) => {
                write!(f, "unable to get known bluetooth devices: {}", error)
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

#[derive(Debug, Args)]
pub struct ListDevicesArgs {
    /// Filter the table output based on given keys.
    #[arg(short, long, value_delimiter = ',')]
    columns: Option<Vec<ListDevicesColumn>>,

    /// Filter the terse output based on given keys.
    #[arg(short, long, value_delimiter = ',')]
    values: Option<Vec<ListDevicesColumn>>,

    /// Filter output based on device status.
    #[arg(short, long)]
    status: Option<DeviceStatus>,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ListDevicesColumn {
    Alias,
    Address,
    Connected,
    Trusted,
    Bonded,
    Paired,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum DeviceStatus {
    Connected,
    Trusted,
    Bonded,
    Paired,
}

const DEFAULT_LISTING_KEYS: [ListDevicesColumn; 6] = [
    ListDevicesColumn::Alias,
    ListDevicesColumn::Address,
    ListDevicesColumn::Connected,
    ListDevicesColumn::Trusted,
    ListDevicesColumn::Bonded,
    ListDevicesColumn::Paired,
];

enum ListDevicesOutput {
    Pretty,
    Terse,
}

pub trait BtListingConverter {
    fn get_listing_field_by_key(&self, value: &ListDevicesColumn) -> String;
    fn filter_listing_by_status(&self, value: &Option<DeviceStatus>) -> bool;
}

impl BtListingConverter for bluez::Device {
    fn get_listing_field_by_key(&self, value: &ListDevicesColumn) -> String {
        match value {
            ListDevicesColumn::Alias => self.alias().to_string(),
            ListDevicesColumn::Address => self.address().to_string(),
            ListDevicesColumn::Connected => self.connected().to_string(),
            ListDevicesColumn::Trusted => self.trusted().to_string(),
            ListDevicesColumn::Bonded => self.bonded().to_string(),
            ListDevicesColumn::Paired => self.paired().to_string(),
        }
    }

    fn filter_listing_by_status(&self, value: &Option<DeviceStatus>) -> bool {
        match value {
            Some(key) => match key {
                DeviceStatus::Connected => self.connected(),
                DeviceStatus::Trusted => self.trusted(),
                DeviceStatus::Bonded => self.bonded(),
                DeviceStatus::Paired => self.paired(),
            },
            None => true,
        }
    }
}

impl From<&ListDevicesColumn> for String {
    fn from(value: &ListDevicesColumn) -> Self {
        let str = match value {
            ListDevicesColumn::Alias => "ALIAS",
            ListDevicesColumn::Address => "ADDRESS",
            ListDevicesColumn::Connected => "CONNECTED",
            ListDevicesColumn::Trusted => "TRUSTED",
            ListDevicesColumn::Bonded => "BONDED",
            ListDevicesColumn::Paired => "PAIRED",
        };

        str.to_string()
    }
}

pub fn list_devices(
    bluez: &crate::BluezClient,
    f: &mut impl io::Write,
    args: &ListDevicesArgs,
) -> Result<(), Error> {
    let (out_format, user_listing_keys) = match (&args.columns, &args.values) {
        (None, None) => (ListDevicesOutput::Pretty, None),
        (None, values) => (ListDevicesOutput::Terse, values.as_ref()),
        (columns, _) => (ListDevicesOutput::Pretty, columns.as_ref()),
    };

    let listing_keys = match user_listing_keys {
        Some(keys) => keys,
        None => &DEFAULT_LISTING_KEYS.to_vec(),
    };

    let devs = bluez.devs().map_err(Error::KnownDevices)?;

    let listing = devs.iter().filter_map(|dev| {
        if !dev.filter_listing_by_status(&args.status) {
            None
        } else {
            Some(
                listing_keys
                    .iter()
                    .map(|k| dev.get_listing_field_by_key(k))
                    .collect::<Vec<String>>(),
            )
        }
    });

    let out_buf = match out_format {
        ListDevicesOutput::Pretty => create_pretty_out(listing, listing_keys),
        ListDevicesOutput::Terse => create_terse_out(listing),
    };

    f.write_all(out_buf.as_bytes())?;

    Ok(())
}

pub fn create_pretty_out(
    listing: impl Iterator<Item = Vec<String>>,
    columns: &[ListDevicesColumn],
) -> String {
    let mut builder = Builder::default();

    builder.push_record(columns);
    for row in listing {
        builder.push_record(row);
    }

    let mut table = builder.build();
    table.with(Style::blank());

    format!("{}", table)
}

pub fn create_terse_out(listing: impl Iterator<Item = Vec<String>>) -> String {
    listing
        .map(|l| {
            let mut terse_str = l.join("/");
            terse_str.push('\n');
            terse_str
        })
        .collect()
}
