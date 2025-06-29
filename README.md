# `bt`


[![CI](https://github.com/acikgozb/bt/actions/workflows/ci.yml/badge.svg)](https://github.com/acikgozb/bt/actions/workflows/ci.yml) ![version](https://img.shields.io/badge/version-0.1.0-red) ![release](https://img.shields.io/badge/release-stable-89e051)

An alternative frontend to `bluetoothctl` which uses Bluez D-Bus to manage Bluetooth connections on the host.

This is a more robust version of the [`devtools/bluetooth`](https://github.com/acikgozb/devtools/blob/main/de/bluetooth) PoC script.

## Table of Contents 

<!--toc:start-->
  - [Installation](#installation)
    - [Build From Source](#build-from-source)
    - [Prebuilt Binaries](#prebuilt-binaries)
  - [Usage](#usage)
    - [`bt status`](#bt-status)
    - [`bt toggle`](#bt-toggle)
    - [`bt list-devices`](#bt-list-devices)
    - [`bt scan`](#bt-scan)
    - [`bt connect`](#bt-connect)
    - [`bt disconnect`](#bt-disconnect)
  - [LICENSE](#license)
<!--toc:end-->

## <a id='installation'></a> Installation

Since `bt` uses Bluez D-Bus, it is designed to be installed on Linux hosts.

The prebuilt binary can be used for `x86_64` Linux hosts. For `arm64`, manual installation can be done instead.

Before proceeding with the installation, ensure that the bluez package is installed on the host (links are for `x86_64`):

- Arch Linux: [bluez](https://archlinux.org/packages/extra/x86_64/bluez/) or [bluez-utils](https://archlinux.org/packages/extra/x86_64/bluez-utils/)
- Fedora: [bluez](https://packages.fedoraproject.org/pkgs/bluez/bluez/)
- Ubuntu: [bluez system snap](https://documentation.ubuntu.com/core/explanation/system-snaps/bluetooth/)
 
There are 2 ways to install `bt`.

### <a id='build-from-source'></a> Build From Source

If you have `cargo` installed on your host, you can use it to build `bt` from source.

```bash
# Clone the repository.
git clone git@github.com:acikgozb/bt.git ./bt

# Install via `cargo`.
cd ./bt
cargo build --release --locked 

# Put the binary under $PATH.
# In here, it is assumed that ~/.local/bin is on $PATH.
cp ./target/release/bt ~/.local/bin/bt

# Validate the $PATH lookup before using hpm.
which bt
```

### <a id='prebuilt-binaries'></a> Prebuilt Binaries

You can also install `bt` by downloading prebuilt binaries from the [releases page](https://github.com/acikgozb/bt/releases).

Extract `bt` from its archive, and then put it under `$PATH` like above.

## <a id='usage'></a> Usage

`bt` provides the subcommands below:

- `status`
- `toggle`
- `list-devices`
- `scan`
- `connect`
- `disconnect`

To understand more about the interface, please refer to `help`:

```bash
bt -h | --help
```

### <a id='bt-status'></a> `bt status`

Use `status` (alias `s`) to get information about the current status of Bluetooth.

```bash
# w/ out any connected devices.
bt status
# bluetooth: enabled
# connected devices:

# w/ connected devices.
bt s
# bluetooth: enabled
# connected devices:
# Dev1/XX:XX:XX:XX:XX:XX (battery: %50)
# Dev2/XX:XX:XX:XX:XX:XX (battery: %55)
# Dev3/XX:XX:XX:XX:XX:XX (battery: %42)
# ...
```

### <a id='bt-toggle'></a> `bt toggle`

Use `toggle` (alias `t`) to toggle the Bluetooth adapter.

```bash
# Assume that Bluetooth is enabled.
bt toggle
# bluetooth: disabled

# Assume that Bluetooth is disabled.
bt t
# bluetooth: enabled
```

### <a id='bt-list-devices'></a> `bt list-devices`

Use `list-devices` (alias `ls`) to see the known Bluetooth devices on the host.

By default, the output is shown in pretty (table) format.

```bash
$ bt list-devices
# ALIAS    ADDRESS             CONNECTED   TRUSTED   BONDED   PAIRED
# Dev1     XX:XX:XX:XX:XX:XX   false       true      false    false
# Dev2     XX:XX:XX:XX:XX:XX   false       true      false    false
```

Similar to `nmcli`, the output can be filtered by specifying which columns you want via `-c | --columns`.

```bash
$ bt ls --columns alias,connected
# ALIAS    CONNECTED
# Dev1     false
# Dev2     false
```

Similar to `nmcli`, a terse output can be printed for scripting purposes by specifying columns you want via `-v | --values`. The fields are separated by `/`.
```bash
$ bt ls --values alias,connected
# Dev1/false
# Dev2/false
```

Additonally, the list can be filtered by specifying the status of the devices you want to see.

In this example, `bt` shows the alias and address of trusted devices only. As you can see, filtering by status does not require that status to exist on the output.

```bash
$ bt ls --columns alias,address --status trusted 
# ALIAS    ADDRESS
# Dev1     XX:XX:XX:XX:XX:XX
# Dev2     XX:XX:XX:XX:XX:XX
```

### <a id='bt-scan'></a> `bt scan`

Use `scan` (alias `sc`) to see the available Bluetooth devices.

By default, the output is shown in pretty (table) format.

```bash
# This is same with `bt scan --columns`.
$ bt scan
# ALIAS    ADDRESS             RSSI
# Dev3     XX:XX:XX:XX:XX:XX   -92
```

For terse output, `-v | --values` can be used, similar to `bt ls`.

```bash
$ bt sc --values
# Dev3/XX:XX:XX:XX:XX:XX/-97
# Dev4/XX:XX:XX:XX:XX:XX/-78
```

Similar to `bt ls`, the output can be filtered by specifying the corresponding columns.

```bash
$ bt sc --values alias,rssi
# Dev3/-97
# Dev4/-78

$ bt sc --columns alias,rssi
# ALIAS   RSSI
# Dev3    -97
# Dev4    -78
```

Use `-d | --duration` to set the scan duration.
The duration is in seconds. The default is 5 seconds, and the max is 60.

```bash
$ bt sc --duration 10
```

### <a id='bt-connect'></a> `bt connect`

Use `connect` (alias `c`) to connect to an available Bluetooth device. The flow changes based on the arguments:

**Interactive**: If an alias is not provided as an argument, `bt connect` runs interactively. In this mode, it does a scan first to show a list of available devices, and tries to connect the one that is selected by the user.

```bash
$ bt connect
# IDX   ALIAS               ADDRESS             RSSI
# (0)   XX-XX-XX-XX-XX-XX   XX:XX:XX:XX:XX:XX   -94
# (1)   XX-XX-XX-XX-XX-XX   XX:XX:XX:XX:XX:XX   -50
# (2)   XX-XX-XX-XX-XX-XX   XX:XX:XX:XX:XX:XX   -68
# (3)   dummy-device        XX:XX:XX:XX:XX:XX   -80
# Select the device you wish to connect:
```

In the interactive mode, you can set the scan duration by using `-d | --duration`, similar to `bt scan`. By default, the scan duration is 5 seconds:

```bash
$ bt c --duration 10
# IDX   ALIAS               ADDRESS             RSSI
# (0)   XX-XX-XX-XX-XX-XX   XX:XX:XX:XX:XX:XX   -94
# (1)   XX-XX-XX-XX-XX-XX   XX:XX:XX:XX:XX:XX   -50
# (2)   XX-XX-XX-XX-XX-XX   XX:XX:XX:XX:XX:XX   -68
# (3)   dummy-device        XX:XX:XX:XX:XX:XX   -80
# Select the device you wish to connect:
```

In interactive mode, you can use `-c | --contains-name` to filter the scan result by device ALIAS'es.
The value used for `-c | --contains-name` should be a simple substring of the device ALIAS. Regex is not supported as of now.

```bash
$ bt c --contains-name dummy
# IDX   ALIAS                   ADDRESS             RSSI
# (0)   dummy-device            XX:XX:XX:XX:XX:XX   -80
# (1)   another-dummy-device    XX:XX:XX:XX:XX:XX   -54
# Select the device you wish to connect:
```

**Non-interactive**: If an alias is provided as an argument, `bt connect` skips the scan and tries to connect to the device directly.

```bash
# Provide the known device's ALIAS to directly connect to it.
$ bt connect <KNOWN_DEVICE_ALIAS>
```

If an ALIAS is provided along with the flags used in the interactive mode, the ALIAS takes precedence and `bt connect` runs non-interactively.

### <a id='bt-disconnect'></a> `bt disconnect`

Use `disconnect` (alias `d`) to disconnect from a connected device. The flow changes based on the arguments.


**Interactive**: If an alias is not provided as an argument, `bt disconnect` runs interactively, similar to `bt connect`. In this mode, it first shows a list of connected devices, and tries to disconnect from the one that is selected by the user.

In order to disconnect from multiple devices, specify the indexes as a comma-separated list.

```bash
$ bt disconnect
# IDX   ALIAS   ADDRESS
# (0)   dev1    XX:XX:XX:XX:XX:XX
# (1)   dev2    XX:XX:XX:XX:XX:XX
# Select the device(s) you wish to disconnect: 0,1
# disconnected from dev1
# disconnected from dev2
```

**Non-interactive**: If an alias is provided as an argument, `bt disconnect` skips showing the connected devices and tries to disconnect from the device(s) directly.

```bash
$ bt d dev1,dev2
# disconnected from dev1
# disconnected from dev2
```

`bt disconnect` can be used to remove a device as well, by specifying `-f | --force`.

This flag can be used in both interactive and non-interactive modes.

```bash
$ bt d --force dev1
# removed device dev1 (forced)
```

## <a id='license'></a> LICENSE

This work is dual-licensed under Apache 2.0 and GPL 2.0 (or any later version).
You can choose between one of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR GPL-2.0-or-later`
