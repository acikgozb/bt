use std::{collections::BTreeMap, error, io};

use tabled::{builder::Builder, settings::Style};

use crate::bluez;

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
) -> Result<(), Box<dyn error::Error>> {
    let bluez = bluez::Client::new()?;

    let aliases = match aliases.as_ref() {
        Some(aliases) => aliases,
        None => &{ get_aliases_from_user(w, r, &bluez)? },
    };

    for alias in aliases {
        let alias = alias.trim();

        let disconnect_result = if *force {
            bluez.remove(alias)?;
            format!("removed device {} (forced)\n", alias)
        } else {
            bluez.disconnect(alias)?;
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
) -> Result<Vec<String>, Box<dyn error::Error>> {
    let devices = bluez.connected_devs()?;
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
        // WARN: Once the errors are designed, replace this unwrap call accordingly.
        let device = device_map.remove(&(idx as usize)).unwrap();
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
