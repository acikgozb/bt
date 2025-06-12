```rust
        // let devices = result
        //     .into_iter()
        //     .filter_map(|(object_path, object)| {
        //         if let Some(path) = object_path.rsplitn(2, "/").take(1).next() {
        //             if !path.contains("dev") {
        //                 None
        //             } else {
        //                 Some(object)
        //             }
        //         } else {
        //             None
        //         }
        //     })
        //     .filter_map(|mut object| {
        //         let mut dev_iface = object.remove("org.bluez.Device1")?;

        //         let conn_status = dev_iface.remove("Connected")?;
        //         let is_connected = bool::try_from(conn_status).ok()?;
        //         if !is_connected {
        //             return None;
        //         }

        //         let alias = String::try_from(dev_iface.remove("Alias")?).ok()?;
        //         let address = String::try_from(dev_iface.remove("Address")?).ok()?;

        //         let mut batt_iface = object.remove("org.bluez.Battery1")?;
        //         let battery = u8::try_from(batt_iface.remove("Percentage")?).ok()?;

        //         Some(BluezConnectedDevStatus {
        //             alias,
        //             address,
        //             battery,
        //         })
        //     })
        //     .collect::<Vec<BluezConnectedDevStatus>>();
```
