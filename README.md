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

## Dependencies [WIP]
WIP

## Build [WIP]
There are no pre-made binaries available to use as hsctl is still in development.<br>
To use hsctl, you can compile it using the instructions below.<br>

If you haven't done so already, install the Rust compiler [here](https://rust-lang.org/tools/install/).<br>

Clone this repository either through GitHub or by running<br>
`git clone https://github.com/oauthul/hsctl.git`<br>

Install the dependencies.<br>

Navigate to the hsctl repository in your favorite terminal and run<br>
`cargo build --release`

