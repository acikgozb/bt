pub mod api;

use std::fmt::Debug;
use std::{error, fmt, io};
use tabled::builder::Builder;
use tabled::settings::Style;
use zbus::blocking::Connection;
use zbus::blocking::fdo::ObjectManagerProxy;
use zbus::proxy;
use zbus::zvariant::OwnedObjectPath;

#[proxy(
    default_service = "org.bluez",
    default_path = "/org/bluez/hci0",
    interface = "org.bluez.Adapter1",
    gen_blocking = true,
    blocking_name = "BluezAdapterProxy",
    async_name = "BluezAsyncAdapterProxy"
)]
pub trait BluezAdapter {
    #[zbus(property, name = "PowerState")]
    fn power_state(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn set_powered(&self, power_state: bool) -> zbus::Result<()>;
}

#[proxy(
    default_service = "org.bluez",
    interface = "org.bluez.Device1",
    gen_blocking = true,
    blocking_name = "BluezDeviceProxy",
    async_name = "BluezAsyncDeviceProxy"
)]
pub trait BluezDevice {
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn bonded(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn paired(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn trusted(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn alias(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;
}

#[proxy(
    default_service = "org.bluez",
    interface = "org.bluez.Battery1",
    gen_blocking = true,
    blocking_name = "BluezDeviceBatteryProxy",
    async_name = "BluezAsyncDeviceBatteryProxy"
)]
pub trait BluezDeviceBattery {
    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<u8>;
}

pub enum BluezPowerState {
    On,
    Off,
}
impl fmt::Display for BluezPowerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            BluezPowerState::On => write!(f, "enabled"),
            BluezPowerState::Off => write!(f, "disabled"),
        }
    }
}

impl From<String> for BluezPowerState {
    fn from(value: String) -> Self {
        if &value == "on" {
            BluezPowerState::On
        } else {
            BluezPowerState::Off
        }
    }
}

impl std::ops::Not for BluezPowerState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            BluezPowerState::On => Self::Off,
            BluezPowerState::Off => Self::On,
        }
    }
}

impl From<&BluezPowerState> for bool {
    fn from(value: &BluezPowerState) -> Self {
        match value {
            BluezPowerState::On => true,
            BluezPowerState::Off => false,
        }
    }
}

#[derive(Debug)]
pub struct BluezDev {
    alias: String,
    address: String,
    connected: bool,
    paired: bool,
    trusted: bool,
    bonded: bool,
    battery: Option<u8>,
}

pub struct Bluez {
    connection: Connection,
}

impl Bluez {
    pub fn new() -> zbus::Result<Self> {
        let connection = Connection::system()?;
        Ok(Self { connection })
    }

    fn get_dev_object_paths(&self) -> zbus::Result<Vec<OwnedObjectPath>> {
        let object_manager_proxy = ObjectManagerProxy::new(&self.connection, "org.bluez", "/")?;
        let objects = object_manager_proxy.get_managed_objects()?;

        let dev_paths = objects
            .into_keys()
            .filter(|k| {
                if let Some(path) = k.rsplitn(2, "/").take(1).next() {
                    path.contains("dev")
                } else {
                    false
                }
            })
            .collect::<Vec<OwnedObjectPath>>();

        Ok(dev_paths)
    }

    fn build_proxy<'a, T>(&self, path: Option<&'a str>) -> zbus::Result<T>
    where
        T: zbus::blocking::proxy::ProxyImpl<'a> + From<zbus::Proxy<'a>>,
    {
        let mut proxy_builder = T::builder(&self.connection);

        if let Some(path) = path {
            proxy_builder = proxy_builder.path(path)?;
        }

        proxy_builder.build()
    }

    pub fn power_state(&self) -> zbus::Result<BluezPowerState> {
        let adapter_proxy: BluezAdapterProxy = self.build_proxy(None)?;
        let result = adapter_proxy.power_state().map(BluezPowerState::from)?;

        Ok(result)
    }

    pub fn toggle_power_state(&self) -> zbus::Result<BluezPowerState> {
        let adapter_proxy: BluezAdapterProxy = self.build_proxy(None)?;
        let prev_state = adapter_proxy.power_state().map(BluezPowerState::from)?;

        let new_state = !prev_state;
        adapter_proxy.set_powered(bool::from(&new_state))?;

        Ok(new_state)
    }

    pub fn devs(&self) -> zbus::Result<Vec<BluezDev>> {
        let dev_paths = self.get_dev_object_paths()?;

        Ok(dev_paths
            .into_iter()
            .filter_map(|dev_path| {
                let dev_proxy: BluezDeviceProxy = self.build_proxy(Some(&dev_path)).ok()?;

                let mut dev = BluezDev {
                    alias: dev_proxy.alias().ok()?,
                    address: dev_proxy.address().ok()?,
                    connected: dev_proxy.connected().ok()?,
                    paired: dev_proxy.paired().ok()?,
                    trusted: dev_proxy.trusted().ok()?,
                    bonded: dev_proxy.bonded().ok()?,
                    battery: None,
                };

                if !dev.connected {
                    return Some(dev);
                }

                let battery_proxy: BluezDeviceBatteryProxy =
                    self.build_proxy(Some(&dev_path)).ok()?;
                dev.battery = Some(battery_proxy.percentage().ok()?);

                Some(dev)
            })
            .collect::<Vec<BluezDev>>())
    }

    pub fn connected_devs(&self) -> zbus::Result<Vec<BluezDev>> {
        let devs = self.devs()?;

        Ok(devs.into_iter().filter(|d| d.connected).collect())
    }
}

pub fn status(f: &mut impl io::Write) -> Result<(), Box<dyn error::Error>> {
    let bluez = Bluez::new()?;

    let power_state = bluez.power_state()?;
    let connected_devs = bluez.connected_devs()?;

    let mut buf = [
        "bluetooth: ",
        &power_state.to_string(),
        "\nconnected devices: ",
    ]
    .join("");
    for dev in connected_devs {
        let format = format!(
            "\n{}/{} (batt: %{})",
            dev.alias,
            dev.address,
            dev.battery.unwrap()
        );
        buf.push_str(&format)
    }

    f.write_all(buf.as_bytes())?;

    Ok(())
}

pub fn toggle(f: &mut impl io::Write) -> Result<(), Box<dyn error::Error>> {
    let bluez = Bluez::new()?;
    let toggled_power_state = bluez.toggle_power_state()?;

    let buf = format!("bluetooth: {}", toggled_power_state);
    f.write_all(buf.as_bytes())?;

    Ok(())
}

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

// TODO: Move this under API crate.
#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum BtListingKey {
    Alias,
    Address,
    Connected,
    Trusted,
    Bonded,
    Paired,
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

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum BtListingStatusKey {
    Connected,
    Trusted,
    Bonded,
    Paired,
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
