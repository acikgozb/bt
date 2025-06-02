use std::{error, io};

use tabled::{builder::Builder, settings::Style};

use crate::{
    api::{BtListingKey, BtListingStatusKey},
    bluez,
};

const DEFAULT_LISTING_KEYS: [BtListingKey; 6] = [
    BtListingKey::Alias,
    BtListingKey::Address,
    BtListingKey::Connected,
    BtListingKey::Trusted,
    BtListingKey::Bonded,
    BtListingKey::Paired,
];

enum BtListingOutput {
    Pretty,
    Terse,
}

pub trait BtListingConverter {
    fn get_listing_field_by_key(&self, value: &BtListingKey) -> String;
    fn filter_listing_by_status(&self, value: &Option<BtListingStatusKey>) -> bool;
}

impl BtListingConverter for bluez::Device {
    fn get_listing_field_by_key(&self, value: &BtListingKey) -> String {
        match value {
            BtListingKey::Alias => self.alias(),
            BtListingKey::Address => self.address(),
            BtListingKey::Connected => self.connected().to_string(),
            BtListingKey::Trusted => self.trusted().to_string(),
            BtListingKey::Bonded => self.bonded().to_string(),
            BtListingKey::Paired => self.paired().to_string(),
        }
    }

    fn filter_listing_by_status(&self, value: &Option<BtListingStatusKey>) -> bool {
        match value {
            Some(key) => match key {
                BtListingStatusKey::Connected => self.connected(),
                BtListingStatusKey::Trusted => self.trusted(),
                BtListingStatusKey::Bonded => self.bonded(),
                BtListingStatusKey::Paired => self.paired(),
            },
            None => true,
        }
    }
}

impl From<&BtListingKey> for String {
    fn from(value: &BtListingKey) -> Self {
        let str = match value {
            BtListingKey::Alias => "ALIAS",
            BtListingKey::Address => "ADDRESS",
            BtListingKey::Connected => "CONNECTED",
            BtListingKey::Trusted => "TRUSTED",
            BtListingKey::Bonded => "BONDED",
            BtListingKey::Paired => "PAIRED",
        };

        str.to_string()
    }
}

pub fn list_devices(
    f: &mut impl io::Write,
    columns: Option<Vec<BtListingKey>>,
    values: Option<Vec<BtListingKey>>,
    status: Option<BtListingStatusKey>,
) -> Result<(), Box<dyn error::Error>> {
    let (out_format, user_listing_keys) = match (columns, values) {
        (None, None) => (BtListingOutput::Pretty, None),
        (None, values) => (BtListingOutput::Terse, values),
        (columns, _) => (BtListingOutput::Pretty, columns),
    };

    let listing_keys = match user_listing_keys {
        Some(keys) => keys,
        None => DEFAULT_LISTING_KEYS.to_vec(),
    };

    let bluez = bluez::Client::new()?;
    let devs = bluez.devs()?;

    let listing = devs.iter().filter_map(|dev| {
        if !dev.filter_listing_by_status(&status) {
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
        BtListingOutput::Pretty => create_pretty_out(listing, &listing_keys),
        BtListingOutput::Terse => create_terse_out(listing),
    };

    f.write_all(out_buf.as_bytes())?;

    Ok(())
}

pub fn create_pretty_out(
    listing: impl Iterator<Item = Vec<String>>,
    columns: &[BtListingKey],
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
