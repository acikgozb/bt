use std::{error, io};

use crate::bluez;

pub fn status(f: &mut impl io::Write) -> Result<(), Box<dyn error::Error>> {
    let bluez = bluez::Client::new()?;

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
            dev.alias(),
            dev.address(),
            dev.battery().unwrap()
        );
        buf.push_str(&format)
    }

    f.write_all(buf.as_bytes())?;

    Ok(())
}
