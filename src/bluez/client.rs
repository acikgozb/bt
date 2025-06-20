use std::{error, fmt};

use zbus::{
    blocking::{Connection, fdo::ObjectManagerProxy},
    zvariant::OwnedObjectPath,
};

use super::proxies::{BluezAdapterProxy, BluezDeviceBatteryProxy, BluezDeviceProxy};

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
    rssi: Option<i16>,
}
impl BluezDev {
    pub fn connected(&self) -> bool {
        self.connected
    }

    pub fn paired(&self) -> bool {
        self.paired
    }

    pub fn trusted(&self) -> bool {
        self.trusted
    }

    pub fn bonded(&self) -> bool {
        self.bonded
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn battery(&self) -> &Option<u8> {
        &self.battery
    }

    pub fn rssi(&self) -> &Option<i16> {
        &self.rssi
    }
}

#[derive(Debug)]
pub enum Error {
    DBusClient(zbus::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DBusClient(error) => {
                write!(f, "unable to establish a Bluez D-Bus connection: {}", error)
            }
        }
    }
}
impl error::Error for Error {}

impl From<zbus::Error> for Error {
    fn from(value: zbus::Error) -> Self {
        Self::DBusClient(value)
    }
}

pub struct Bluez {
    connection: Connection,
    adapter_proxy: BluezAdapterProxy<'static>,
}

impl Bluez {
    pub fn new() -> Result<Self, Error> {
        let connection = Connection::system()?;
        let adapter_proxy = BluezAdapterProxy::new(&connection)?;

        Ok(Self {
            connection,
            adapter_proxy,
        })
    }

    fn dev_object_iter(&self) -> zbus::Result<impl Iterator<Item = OwnedObjectPath>> {
        let object_manager_proxy = ObjectManagerProxy::new(&self.connection, "org.bluez", "/")?;
        let objects = object_manager_proxy.get_managed_objects()?;

        let dev_paths = objects.into_keys().filter(|k| {
            if let Some(path) = k.rsplitn(2, "/").take(1).next() {
                path.contains("dev")
            } else {
                false
            }
        });

        Ok(dev_paths)
    }

    // FIXME: No need for this at all, use proxy::new in approp places.
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
        let result = self
            .adapter_proxy
            .power_state()
            .map(BluezPowerState::from)?;

        Ok(result)
    }

    pub fn toggle_power_state(&self) -> zbus::Result<BluezPowerState> {
        let prev_state = self
            .adapter_proxy
            .power_state()
            .map(BluezPowerState::from)?;

        let new_state = !prev_state;
        self.adapter_proxy.set_powered(bool::from(&new_state))?;

        Ok(new_state)
    }

    pub fn devs(&self) -> zbus::Result<Vec<BluezDev>> {
        let dev_object_iter = self.dev_object_iter()?;

        Ok(dev_object_iter
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
                    rssi: None,
                };

                if let Ok(rssi) = dev_proxy.rssi() {
                    dev.rssi = Some(rssi);
                }

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

    pub fn connect(&self, alias: &str) -> zbus::Result<()> {
        let dev_paths = self.dev_object_iter()?;

        for dev_path in dev_paths {
            let dev_proxy: BluezDeviceProxy = self.build_proxy(Some(&dev_path))?;

            let dev_alias = dev_proxy.alias()?;
            if dev_alias == alias {
                return dev_proxy.connect();
            }
        }

        Err(zbus::Error::InterfaceNotFound)
    }

    pub fn connected_devs(&self) -> zbus::Result<Vec<BluezDev>> {
        let devs = self.devs()?;

        Ok(devs.into_iter().filter(|d| d.connected).collect())
    }

    pub fn start_discovery(&self) -> zbus::Result<()> {
        self.adapter_proxy.start_discovery()
    }

    pub fn stop_discovery(&self) -> zbus::Result<()> {
        self.adapter_proxy.stop_discovery()
    }

    pub fn scanned_devices(&self) -> zbus::Result<Vec<BluezDev>> {
        let devs = self.devs()?;
        Ok(devs.into_iter().filter(|d| d.rssi.is_some()).collect())
    }

    pub fn remove(&self, alias: &str) -> zbus::Result<()> {
        let mut dev_object_iter = self.dev_object_iter()?;

        let dev_object = dev_object_iter.find_map(|obj| {
            let dev_object = obj.into_inner();
            let dev_proxy = BluezDeviceProxy::new(&self.connection, &dev_object).ok()?;

            if alias == dev_proxy.alias().ok()? {
                Some(dev_object)
            } else {
                None
            }
        });

        if let Some(dev_object) = dev_object {
            self.adapter_proxy.remove_device(dev_object)
        } else {
            Err(zbus::Error::InterfaceNotFound)
        }
    }

    pub fn disconnect(&self, alias: &str) -> zbus::Result<()> {
        let mut dev_object_iter = self.dev_object_iter()?;

        let dev_proxy = dev_object_iter.find_map(|obj| {
            let dev_object = obj.into_inner();
            let dev_proxy = BluezDeviceProxy::new(&self.connection, &dev_object).ok()?;

            if alias == dev_proxy.alias().ok()? {
                Some(dev_proxy)
            } else {
                None
            }
        });

        if let Some(dev_proxy) = dev_proxy {
            dev_proxy.disconnect()
        } else {
            Err(zbus::Error::InterfaceNotFound)
        }
    }
}
