# TODO BT Scan Flow

1 - Find out how a BT scan is done through Bluez DBus.
2 - Create the `bt scan` API.
3 - Create necessary Bluez proxies through `zbus` if necessary.
4 - Use the `--duration` flag to set the scan duration.
5 - Make sure that scan results contain previously connected/bonded/trusted/paired devices. This was a bug in `bluetooth` shell script that forced me to connect by writing the mac address by hand.

## Notes taken during the implementation


### Showing previously connected devices after scan

About #5, it is actually not correct to show previously connected/bonded/trusted/paired devices. Because there may be a case where `bt` shows the devices, but they are actually not connectable.

Instead, `bt scan` should always show newly connected devices instead, like `bluetoothctl` does.

The bug in `bluetooth` shell script can be solved during the `bt connect` implementation.
In there, we can actually show the known devices as well because users tend to connect to their known devices, and before doing so they usually set the device to pairing mode.
 
So, `bt scan` should work exactly like `bluetooth` shell script, in a way that it only shows newly discovered devices that have their aliases set by the user.

If the alias is not set, then Bluez set it to remote device's address, replacing `:` with `-`.

### Showing the scan result to user

In general, `bt` should follow a single standard for showing its output to users.

Since we implemented a table/terse output formatting in `bt ls`, we can continue to do so for `bt scan` as well.
However, this means the formatting logic needs to be extracted from `bt ls` and should be moved under a different module, which should be usable by both `bt ls` and `bt scan`.

The same formatting will be used for `bt connect` in the future as well, or `bt disconnect`.
