#![allow(dead_code, reason = "cfg test/not(test) for BluezDBusClient")]

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

/// Defines a Bluetooth device.
/// It is constructed from [`BluezClient`] methods.
///
/// [`BluezClient`]: crate::BluezClient
#[derive(Debug)]
pub struct BluezDevice {
    alias: String,
    address: String,
    connected: bool,
    paired: bool,
    trusted: bool,
    bonded: bool,
    battery: Option<u8>,
    rssi: Option<i16>,
}
impl BluezDevice {
    /// Indicates whether a [`BluezDevice`] is connected or not.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    pub fn connected(&self) -> bool {
        self.connected
    }

    /// Indicates whether a [`BluezDevice`] is paired or not.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    pub fn paired(&self) -> bool {
        self.paired
    }

    /// Indicates whether a [`BluezDevice`] is trusted or not.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    pub fn trusted(&self) -> bool {
        self.trusted
    }

    /// Indicates whether a [`BluezDevice`] is bonded or not.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    pub fn bonded(&self) -> bool {
        self.bonded
    }

    /// Provides a [`BluezDevice`]'s alias.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// Provides a [`BluezDevice`]'s MAC address.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Provides a [`BluezDevice`]'s battery.
    ///
    /// If a [`BluezDevice`] is connected, then the returned value is [`Some`].
    /// If a [`BluezDevice`] is not connected, then the returned value is [`None`].
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    /// [`Some`]: std::option::Option::Some
    /// [`None`]: std::option::Option::None
    pub fn battery(&self) -> &Option<u8> {
        &self.battery
    }

    /// Provides a [`BluezDevice`]'s RSSI.
    ///
    /// This value is [`Some`] only when the device emits Bluetooth signals.
    /// Otherwise, it is [`None`].
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    /// [`Some`]: std::option::Option::Some
    /// [`None`]: std::option::Option::None
    pub fn rssi(&self) -> &Option<i16> {
        &self.rssi
    }
}

/// Defines error variants that may be returned from [`BluezClient`].
///
/// [`BluezClient`]: crate::BluezClient
#[derive(Debug, Clone)]
pub enum Error {
    /// Happens when there is an error during the initialization of [`BluezClient`]. This error mainly indicates a DBus error on the host.
    ///
    /// It holds the underlying DBus error.
    ///
    /// [`BluezClient`]: crate::BluezClient
    Init(zbus::Error),

    /// Happens when a [`BluezClient`] process fails.
    ///
    /// It holds the process' ID, and the underlying DBus error.
    ///
    /// [`BluezClient`]: crate::BluezClient
    Process(String, zbus::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Init(error) => {
                write!(f, "unable to establish a Bluez D-Bus connection: {}", error)
            }
            Error::Process(pid, error) => {
                write!(f, "the Bluez process '{}' failed: {}", pid, error)
            }
        }
    }
}
impl error::Error for Error {}

/// Defines the client that interacts with Bluez D-Bus.
pub struct BluezDBusClient {
    connection: Connection,
    adapter_proxy: BluezAdapterProxy<'static>,
}

impl BluezDBusClient {
    /// Init method. The initialized [`BluezClient`] can be re-used for multiple processes.
    ///
    /// The error returning from this method is of [`BluezError::Init`] variant.
    ///
    /// [`BluezClient`]: crate::BluezClient
    /// [`BluezError::Init`]: crate::BluezError::Init
    pub fn new() -> Result<Self, Error> {
        let connection = Connection::system().map_err(Error::Init)?;
        let adapter_proxy = BluezAdapterProxy::new(&connection).map_err(Error::Init)?;

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

    /// Provides the power state of the Bluetooth adapter.
    ///
    /// It fails when the power state cannot be read from Bluez D-Bus.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn power_state(&self) -> Result<BluezPowerState, Error> {
        let result = self
            .adapter_proxy
            .power_state()
            .map(BluezPowerState::from)
            .map_err(|e| Error::Process(String::from("power_state"), e))?;

        Ok(result)
    }

    /// Toggles the power state of the Bluetooth adapter.
    ///
    /// It fails when the power state cannot be read, or when Bluez D-Bus fails to set the new power state.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn toggle_power_state(&self) -> Result<BluezPowerState, Error> {
        let prev_state = self.power_state()?;

        let new_state = !prev_state;
        self.adapter_proxy
            .set_powered(bool::from(&new_state))
            .map_err(|e| Error::Process(String::from("toggle_power_state"), e))?;

        Ok(new_state)
    }

    /// Provides the list of [`BluezDevice`]'s registered on the host.
    ///
    /// For the connected devices, each [`BluezDevice.battery()`] returns [`Some`].
    /// For the non-connected devices, each [`BluezDevice.battery()`] returns [`None`].
    /// For the devices that emit Bluetooth signals, each [`BluezDevice.rssi()`] returns [`Some`].
    ///
    /// It fails when the device object paths are cannot be read from Bluez D-Bus.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    /// [`BluezError::Process`]: crate::BluezError::Process
    /// [`Some`]: std::option::Option::Some
    /// [`None`]: std::option::Option::None
    /// [`BluezDevice.battery()`]: crate::BluezDevice::battery()
    /// [`BluezDevice.rssi()`]: crate::BluezDevice::rssi()
    pub fn devices(&self) -> Result<Vec<BluezDevice>, Error> {
        let dev_object_iter = self
            .dev_object_iter()
            .map_err(|e| Error::Process(String::from("devices"), e))?;

        Ok(dev_object_iter
            .filter_map(|dev_path| {
                let dev_proxy = BluezDeviceProxy::new(&self.connection, &dev_path).ok()?;

                let mut dev = BluezDevice {
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

                let battery_proxy =
                    BluezDeviceBatteryProxy::new(&self.connection, &dev_path).ok()?;
                dev.battery = Some(battery_proxy.percentage().ok()?);

                Some(dev)
            })
            .collect::<Vec<BluezDevice>>())
    }

    /// Connects to a Bluetooth device by it's alias.
    ///
    /// It fails if a device cannot be found for the provided alias, or the Bluez D-Bus fails during the connection process.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn connect(&self, alias: &str) -> Result<(), Error> {
        let to_connect_err = |e: zbus::Error| Error::Process(String::from("connect"), e);

        let dev_paths = self.dev_object_iter().map_err(to_connect_err)?;

        for dev_path in dev_paths {
            let dev_proxy =
                BluezDeviceProxy::new(&self.connection, &dev_path).map_err(to_connect_err)?;

            let dev_alias = dev_proxy.alias().map_err(to_connect_err)?;
            if dev_alias == alias {
                return dev_proxy.connect().map_err(to_connect_err);
            }
        }

        Err(to_connect_err(zbus::Error::InterfaceNotFound))
    }

    /// Provides a list of connected [`BluezDevice`]'s.
    ///
    /// Each [`BluezDevice`] has their [`BluezDevice.battery()`] set to [`Some`].
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant. The error cases are the same with [`BluezClient::devices()`].
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    /// [`BluezClient::devices()`]: crate::BluezClient::devices()
    /// [`BluezError::Process`]: crate::BluezError::Process
    /// [`Some`]: std::option::Option::Some
    /// [`BluezDevice.battery()`]: crate::BluezDevice::battery()
    pub fn connected_devices(&self) -> Result<Vec<BluezDevice>, Error> {
        let devs = self.devices()?;

        Ok(devs.into_iter().filter(|d| d.connected).collect())
    }

    /// Starts the device discovery to register available Bluetooth devices to the host.
    ///
    /// Only one discovery session can be alive at a time for each [`BluezClient`] instance.
    ///
    /// In order to stop the device discovery, use [`BluezClient.stop_discovery()`]. It is the caller's responsiblity to stop the device discovery.
    ///
    /// In order to get a list of scanned devices, use [`BluezClient.scanned_devices()`].    
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezClient`]: crate::BluezClient
    /// [`BluezClient.stop_discovery()`]: crate::BluezClient::stop_discovery()
    /// [`BluezClient.scanned_devices()`]: crate::BluezClient::scanned_devices()
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn start_discovery(&self) -> Result<(), Error> {
        self.adapter_proxy
            .start_discovery()
            .map_err(|e| Error::Process(String::from("start_disc"), e))
    }

    /// Stops the device discovery to remove registered available Bluetooth devices from the host and end the scanning process.
    ///
    /// In order to get a list of scanned devices, use [`BluezClient.scanned_devices()`]. It is recommended to get the list of scanned devices before calling this method.    
    ///
    /// It fails when Bluez D-Bus fails to start the discovery.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezClient`]: crate::BluezClient
    /// [`BluezClient.scanned_devices()`]: crate::BluezClient::scanned_devices()
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn stop_discovery(&self) -> Result<(), Error> {
        self.adapter_proxy
            .stop_discovery()
            .map_err(|e| Error::Process(String::from("stop_disc"), e))
    }

    /// Returns a list of scanned [`BluezDevice`]'s. These devices are registered to the host during a device discovery session.
    ///
    /// Each [`BluezDevice`] has their [`BluezDevice.rssi()`] set to [`Some`].
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant. The error cases are the same with [`BluezClient::devices()`].
    ///
    /// [`BluezDevice`]: crate::BluezDevice
    /// [`BluezDevice.rssi()`]: crate::BluezDevice::rssi()
    /// [`BluezClient::devices()`]: crate::BluezClient::devices()
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn scanned_devices(&self) -> Result<Vec<BluezDevice>, Error> {
        let devs = self.devices()?;
        Ok(devs.into_iter().filter(|d| d.rssi.is_some()).collect())
    }

    /// Removes a Bluetooth device from the known device list on the host by it's alias.
    ///
    /// It fails if a device cannot be found for the provided alias, or if Bluez D-Bus fails to remove.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn remove(&self, alias: &str) -> Result<(), Error> {
        let to_remove_err = |e: zbus::Error| Error::Process(String::from("remove"), e);

        let mut dev_object_iter = self.dev_object_iter().map_err(to_remove_err)?;

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
            self.adapter_proxy
                .remove_device(dev_object)
                .map_err(to_remove_err)
        } else {
            Err(to_remove_err(zbus::Error::InterfaceNotFound))
        }
    }

    /// Disconnects a Bluetooth device from the host by it's alias.
    ///
    /// It fails if a device cannot be found for the provided alias, or if Bluez D-Bus fails to disconnect.
    ///
    /// The error returning from this method is of [`BluezError::Process`] variant.
    ///
    /// [`BluezError::Process`]: crate::BluezError::Process
    pub fn disconnect(&self, alias: &str) -> Result<(), Error> {
        let to_disconnect_err = |e: zbus::Error| Error::Process(String::from("disconnect"), e);

        let mut dev_object_iter = self.dev_object_iter().map_err(to_disconnect_err)?;

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
            dev_proxy.disconnect().map_err(to_disconnect_err)
        } else {
            Err(to_disconnect_err(zbus::Error::InterfaceNotFound))
        }
    }
}

pub struct BluezTestClient {
    erred_method_name: Option<String>,
    err: Error,
}

impl BluezTestClient {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            erred_method_name: None,
            err: Error::Process(String::from("test_proc"), zbus::Error::InvalidReply),
        })
    }

    pub fn set_erred_method_name(&mut self, name: String) {
        self.erred_method_name = Some(name);
    }

    pub fn power_state(&self) -> Result<BluezPowerState, Error> {
        let err_key = String::from("power_state");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(BluezPowerState::On),
        }
    }

    pub fn toggle_power_state(&self) -> Result<BluezPowerState, Error> {
        let err_key = String::from("toggle_power_state");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(BluezPowerState::Off),
        }
    }

    pub fn devices(&self) -> Result<Vec<BluezDevice>, Error> {
        let err_key = String::from("devices");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => {
                let device = BluezDevice {
                    alias: String::from("test_dev"),
                    address: String::from("XX:XX:XX:XX:XX:XX"),
                    connected: true,
                    paired: true,
                    trusted: true,
                    bonded: false,
                    battery: Some(50),
                    rssi: None,
                };

                Ok(vec![device])
            }
        }
    }

    pub fn connect(&self, _: &str) -> Result<(), Error> {
        let err_key = String::from("connect");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(()),
        }
    }

    pub fn connected_devices(&self) -> Result<Vec<BluezDevice>, Error> {
        let err_key = String::from("connected_devices");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => {
                let device = BluezDevice {
                    alias: String::from("test_dev"),
                    address: String::from("XX:XX:XX:XX:XX:XX"),
                    connected: true,
                    paired: true,
                    trusted: true,
                    bonded: false,
                    battery: Some(50),
                    rssi: None,
                };

                Ok(vec![device])
            }
        }
    }

    pub fn start_discovery(&self) -> Result<(), Error> {
        let err_key = String::from("start_discovery");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(()),
        }
    }

    pub fn stop_discovery(&self) -> Result<(), Error> {
        let err_key = String::from("stop_discovery");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(()),
        }
    }

    pub fn scanned_devices(&self) -> Result<Vec<BluezDevice>, Error> {
        let err_key = String::from("scanned_devices");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => {
                let device = BluezDevice {
                    alias: String::from("test_dev"),
                    address: String::from("XX:XX:XX:XX:XX:XX"),
                    connected: true,
                    paired: true,
                    trusted: true,
                    bonded: false,
                    battery: None,
                    rssi: Some(50),
                };

                Ok(vec![device])
            }
        }
    }

    pub fn remove(&self, _: &str) -> Result<(), Error> {
        let err_key = String::from("remove");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(()),
        }
    }

    pub fn disconnect(&self, _: &str) -> Result<(), Error> {
        let err_key = String::from("disconnect");

        match &self.erred_method_name {
            Some(v) if v == &err_key => Err(self.err.clone()),
            _ => Ok(()),
        }
    }
}
