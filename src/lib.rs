pub mod api;
use std::fmt::Debug;
use std::{error, fmt, io};
use tabled::builder::Builder;
use tabled::settings::Style;

enum BtListingOutput {
    Pretty,
    Terse,
}

pub trait BtListingConverter {
    fn get_field_with_key(&self, value: &BtListingKey) -> Box<&dyn fmt::Display>;
    fn filter_by_status(&self, value: &Option<BtListingStatusKey>) -> bool;
}

impl BtListingConverter for BluezDev {
    fn get_field_with_key(&self, value: &BtListingKey) -> Box<&dyn fmt::Display> {
        match value {
            BtListingKey::Alias => Box::new(&self.alias),
            BtListingKey::Address => Box::new(&self.address),
            BtListingKey::Connected => Box::new(&self.connected),
            BtListingKey::Trusted => Box::new(&self.trusted),
            BtListingKey::Bonded => Box::new(&self.bonded),
            BtListingKey::Paired => Box::new(&self.paired),
        }
    }

    fn filter_by_status(&self, value: &Option<BtListingStatusKey>) -> bool {
        match value {
            Some(key) => match key {
                BtListingStatusKey::Connected => self.connected,
                BtListingStatusKey::Trusted => self.trusted,
                BtListingStatusKey::Bonded => self.bonded,
                BtListingStatusKey::Paired => self.paired,
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

const DEFAULT_LISTING_KEYS: [BtListingKey; 6] = [
    BtListingKey::Alias,
    BtListingKey::Address,
    BtListingKey::Connected,
    BtListingKey::Trusted,
    BtListingKey::Bonded,
    BtListingKey::Paired,
];

// TODO: Now the time comes to create a module this, along with other subcommands.
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

    let bluez = Bluez::new()?;
    let devs = bluez.devs()?;

    let listing = devs.iter().filter_map(|dev| {
        if !dev.filter_by_status(&status) {
            None
        } else {
            Some(
                listing_keys
                    .iter()
                    .map(|k| dev.get_field_with_key(k).to_string())
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
mod bluez;
mod status;
mod toggle;

pub use status::status;
pub use toggle::toggle;
