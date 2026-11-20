# hsctl [headset control]
A lightweight, cross-platform and performant headset monitoring and management application built in Rust.

## Supported Devices
As of now, hsctl supports the HyperX Cloud Stinger 2 Wireless.<br>

hsctl can:
- Get the battery level of the headset,
- Get the charging status of the headset,
- Get the microphone status of the headset,
- Get and set the auto-shutdown time of the headset

..and [more](docs/COMMANDS.md).

## Dependencies
hsctl relies on the [hidapi](https://github.com/ruabmbua/hidapi-rs) crate.<br>
The hidapi crate needs the libudev development headers to compile, which may not be preinstalled on Linux systems.<br>
To ensure hsctl compiles, install the libudev header files:

Debian / Ubuntu (Tested on Debian 13.6 Trixie ARM64):<br>
`sudo apt install libudev-dev`

Fedora (Tested on Fedora 44 ARM64):<br>
`sudo dnf install systemd-devel`

Arch Linux (Tested on Arch Linux x64):<br>
`sudo pacman -S pkgconf`

## Build [WIP]
There are no pre-made binaries available to use as hsctl is still in development.<br>
To use hsctl, you can compile it using the instructions below.<br>

If you haven't done so already, install the Rust compiler [here](https://rust-lang.org/tools/install/).

Clone this repository either through GitHub or by running: `git clone https://github.com/oauthul/hsctl.git`

Install the dependencies.

Navigate to the hsctl repository in your favorite terminal and run: `cargo build --release`

