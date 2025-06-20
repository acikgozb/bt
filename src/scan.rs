use core::fmt;
use std::{error, io, thread, time::Duration};

use clap::Args;
use tabled::{builder::Builder, settings::Style};

use crate::bluez;

#[derive(Debug)]
pub enum Error {
    Start(bluez::Error),
    Stop(bluez::Error),
    DiscoveredDevices(bluez::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Start(error) => write!(f, "unable to start device discovery: {}", error),
            Error::Stop(error) => write!(f, "unable to stop device discovery: {}", error),
            Error::DiscoveredDevices(error) => {
                write!(f, "unable to get discovered devices: {}", error)
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
pub struct ScanArgs {
    /// Set the duration of the scan.
    #[arg(short, long, default_value_t = 5u8)]
    pub duration: u8,

    /// Filter the pretty output based on given columns.
    ///
    /// If no columns are provided, then the full pretty output is shown to the user.
    #[arg(short, long, value_delimiter = ',', num_args = 0.., default_value = None)]
    pub columns: Option<Vec<ScanColumn>>,

    /// Filter the terse output based on given columns.
    ///
    /// If no columns are provided, then the full terse output is shown to the user.
    #[arg(short, long, value_delimiter = ',', num_args = 0.., default_value = None)]
    pub values: Option<Vec<ScanColumn>>,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum ScanColumn {
    Alias,
    Address,
    Rssi,
}

const DEFAULT_LISTING_KEYS: [ScanColumn; 3] =
    [ScanColumn::Alias, ScanColumn::Address, ScanColumn::Rssi];

enum ScanOutput {
    Pretty,
    Terse,
}

pub trait Listable {
    fn get_listing_field_by_column(&self, value: &ScanColumn) -> String;
}

impl Listable for bluez::Device {
    fn get_listing_field_by_column(&self, value: &ScanColumn) -> String {
        match value {
            ScanColumn::Alias => self.alias().to_string(),
            ScanColumn::Address => self.address().to_string(),
            ScanColumn::Rssi => self.rssi().unwrap_or(0).to_string(),
        }
    }
}

impl From<&ScanColumn> for String {
    fn from(value: &ScanColumn) -> Self {
        let str = match value {
            ScanColumn::Alias => "ALIAS",
            ScanColumn::Address => "ADDRESS",
            ScanColumn::Rssi => "RSSI",
        };

        str.to_string()
    }
}

pub fn scan(
    bluez: &crate::BluezClient,
    f: &mut impl io::Write,
    args: &ScanArgs,
) -> Result<(), Error> {
    let (out_format, listing_keys) = match (&args.columns, &args.values) {
        (None, None) => (ScanOutput::Pretty, &DEFAULT_LISTING_KEYS.to_vec()),
        (None, Some(v)) => (
            ScanOutput::Terse,
            if v.is_empty() {
                &DEFAULT_LISTING_KEYS.to_vec()
            } else {
                v
            },
        ),
        (Some(c), _) => (
            ScanOutput::Pretty,
            if c.is_empty() {
                &DEFAULT_LISTING_KEYS.to_vec()
            } else {
                c
            },
        ),
    };

    bluez.start_discovery().map_err(Error::Start)?;
    thread::sleep(Duration::from_secs(u64::from(args.duration)));

    let scanned_devices = bluez.scanned_devices().map_err(Error::DiscoveredDevices)?;

    let listing = scanned_devices.iter().map(|d| {
        listing_keys
            .iter()
            .map(|k| d.get_listing_field_by_column(k))
            .collect::<Vec<String>>()
    });

    let out_buf = match out_format {
        ScanOutput::Pretty => create_pretty_out(listing, listing_keys),
        ScanOutput::Terse => create_terse_out(listing),
    };

    f.write_all(out_buf.as_bytes())?;

    bluez.stop_discovery().map_err(Error::Stop)?;

    Ok(())
}

pub fn create_pretty_out(
    listing: impl Iterator<Item = Vec<String>>,
    columns: &[ScanColumn],
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
