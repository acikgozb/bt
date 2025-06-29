mod client;
mod proxies;

pub use client::{BluezDevice, Error};

#[cfg(not(test))]
pub use client::BluezDBusClient as Client;

#[cfg(test)]
pub use client::BluezTestClient as Client;
