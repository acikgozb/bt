# TODO `bt` API

Here is the current features that are offered by `bluetooth`:

- Enable/Disable bluetooth - done
- Scan devices - done
- Connect to a specific device from scan results, with rescans - done
- Disconnect from device(s) - done

And when `bluetooth` first runs, it shows the status:

- BT on or off - done
- Connected devices if any. - done

So based on this, here is the API:

## Status - done

```bash
$ bt status
bluetooth: enabled/disabled
connected device(s): 
dev1:addr1 (battery: %percentage1)
dev2:addr2 (battery: %percentage2)
```

The status is really straightforward, I need to see the bluetooth driver status, along with the connected devices if any.

## Toggle - done

```bash
$ bt toggle
# Bluetooth status: disabled

$ bt toggle
# Bluetooth status: enabled
```

For the toggle, it's pretty easy as well. It should just switch the BT status and show the updated status to the user.

## List Devices - done

```bash
$ bt ls
DEVICE ADDRESS CONNECTED TRUSTED PAIRED BONDED
dev1   xxxxxxx   false     true    false   false

$ bt ls -c device,connected
DEVICE CONNECTED
dev1     false

$ bt ls -v device,connected
dev1/false

$ bt ls -s trusted|paired|connected|bonded # filter output based on status
```

List devices command can be used to see the status of the available devices on the host.
Like `nmcli`, it can show a table or a terse output if needed.

The output should be able to show filtered output.

This is probably the hardest command of `bt`, but lets see how it goes.

## Scan

```bash
$ bt scan
# Dev1
# Dev2
# Dev3

$ bt scan --duration 10
# Shows results after 10 secs.
```

Scan needs to show a list of available devices that can be connected later on.
Optionally, it may include a `--duration` flag to scan for X seconds.
The default duration may be something like 5 seconds.

## Connect

```bash
$ bt connect
# (0) Dev1
# (1) Dev2
# (2) Dev3
# Select the device you wish to connect (enter s to refresh the list):

$ bt connect --duration 10
# Shows results after 10 secs.
```

Now, the actual connection happens via MAC addresses, but making users write the MAC addresses is not really intuitive.
Therefore, connect should not require MAC addresses, it should be interactive and show the users a scan result to let them choose from.
The underlying implementation should use the corresponding MAC addresses.

It should also be able to accept the same duration flag that is used for `bt scan`.

Now, there was a bug in `bluetooth`, which was actually a feature of `bluetoothctl`.

`bluetoothctl` scan does not include the currently known devices that are also broadcasting. So `bt` should add the known devices to the list along with the scan results.

## Disconnect

```bash
$ bt disconnect
# (0) Dev1
# (1) Dev2
# (2) Dev3
# Select the device to disconnect:

$ bt disconnect -f
# Same with above but deletes the dev from the list of known devices.
```

Disconnect is pretty straightforward.
It should accept a selection from the user (interactive), and then disconnect from it.
It may additionally have a `--force` flag to remove the device from the list of known devices after disconnecting from it.
