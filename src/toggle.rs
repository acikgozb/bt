use std::{error, io};

use crate::bluez;

pub fn toggle(f: &mut impl io::Write) -> Result<(), Box<dyn error::Error>> {
    let bluez = bluez::Client::new()?;
    let toggled_power_state = bluez.toggle_power_state()?;

    let buf = format!("bluetooth: {}", toggled_power_state);
    f.write_all(buf.as_bytes())?;

    Ok(())
}
