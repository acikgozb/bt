mod client;
mod proxies;

pub use client::{BluezDev as Device, Error};

#[cfg(not(test))]
pub use client::BluezDBusClient as Client;

#[cfg(test)]
pub use client::BluezTestClient as Client;
