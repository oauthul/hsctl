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

## Prerequisites

Before building hsctl, ensure you have the right dependencies for your system.

### Linux
hsctl relies on the [hidapi](https://github.com/ruabmbua/hidapi-rs) crate which needs the libudev development headers to compile and may not be preinstalled on Linux systems.

Ensure the libudev development headers are installed for your system:

**Debian / Ubuntu** (tested on Debian 13.6 Trixie ARM64)<br>
```sudo apt install libudev-dev```

**Fedora** (tested on Fedora 44 ARM64)<br>
`sudo dnf install systemd-devel`

**Arch Linux** (tested on Arch Linux x64)<br>
`sudo pacman -S pkgconf`

### Windows
If you're using Windows, you'll need the Visual Studio build tools to install the Rust compiler.

Rust has a [guide](https://rust-lang.github.io/rustup/installation/windows-msvc.html) for this.

### macOS
macOS users do not have any prerequisites.

## Build
There are no pre-made binaries available to use as hsctl is still in development.

To use hsctl, you can compile it using the instructions below.

### Installing the Rust compiler
If you haven't done so already, install the Rust compiler [here](https://rust-lang.org/tools/install/).<br>
At minimum, you need Rust 1.92.0.

### Cloning the repository
Clone the repository:
- Git: `git clone https://github.com/oauthul/hsctl.git`
- GitHub: Download the repository [zip](https://github.com/oauthul/hsctl/archive/refs/heads/main.zip) file

### Building the binary
Run `cargo build --release` in the hsctl directory.

By default, the binary will be located at `<hsctl_root>/target/release/hsctl` on Linux and macOS systems, or `<hsctl_root>/target/release/hsctl.exe` on Windows systems.
