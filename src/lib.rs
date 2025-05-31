pub mod api;

use std::{error, fmt, io};
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
pub struct BluezConnectedDev {
    alias: String,
    address: String,
    battery: u8,
}

impl fmt::Display for BluezConnectedDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} (battery: %{})",
            self.alias, self.address, self.battery
        )
    }
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

    pub fn connected_devs(&self) -> zbus::Result<Vec<BluezConnectedDev>> {
        let dev_paths = self.get_dev_object_paths()?;

        Ok(dev_paths
            .into_iter()
            .filter_map(|dev_path| {
                let dev_proxy: BluezDeviceProxy = self.build_proxy(Some(&dev_path)).ok()?;

                let is_connected = dev_proxy.connected().ok()?;
                if !is_connected {
                    return None;
                }

                let battery_proxy: BluezDeviceBatteryProxy =
                    self.build_proxy(Some(&dev_path)).ok()?;

                let address = dev_proxy.address().ok()?;
                let alias = dev_proxy.alias().ok()?;
                let battery = battery_proxy.percentage().ok()?;

                Some(BluezConnectedDev {
                    alias,
                    address,
                    battery,
                })
            })
            .collect::<Vec<BluezConnectedDev>>())
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
        buf.push('\n');
        buf.push_str(&dev.to_string())
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
