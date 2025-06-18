use zbus::{proxy, zvariant::ObjectPath};

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

    fn start_discovery(&self) -> zbus::Result<()>;

    fn stop_discovery(&self) -> zbus::Result<()>;

    fn remove_device(&self, object: ObjectPath<'static>) -> zbus::Result<()>;
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

    #[zbus(property, name = "RSSI")]
    fn rssi(&self) -> zbus::Result<i16>;

    fn connect(&self) -> zbus::Result<()>;

    fn disconnect(&self) -> zbus::Result<()>;
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
