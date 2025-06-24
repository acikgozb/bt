mod client;
mod proxies;

pub use client::BluezDev as Device;

#[cfg(not(test))]
pub use client::BluezDBusClient as Client;

#[cfg(test)]
pub use client::BluezTestClient as Client;

pub use zbus::Error;
