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

