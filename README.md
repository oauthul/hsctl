# hsctl [headset control]
A lightweight, cross-platform and performant headset monitoring and management application built in Rust.

## OS Compatibility
hsctl is aimed to be cross-platform (Linux, Windows and macOS).

|        OS       | x86 | x64 | ARM | ARM64 |           Tested Version           |
|:---------------:|:---:|:---:|:---:|:-----:|:----------------------------------:|
| Debian / Ubuntu |  ?  |  ?  |  ?  |   ✓   |     Debian 13.6 (trixie) ARM64     |
|      Fedora     |  ?  |  ?  |  ?  |   ✓   |           Fedora 44 ARM64          |
|    Arch Linux   |  ?  |  ✓  |  ?  |   ?   |           Arch Linux x64           |
|     Windows     |  ?  |  ✓  |  ?  |   ✓   | Windows 11 26H2 (26300.9974) ARM64 |
|      macOS      |  ?  |  ?  |  ?  |   ✓   |      macOS Tahoe 26.6.2 ARM64      |

A tick (✓) shows that hsctl was tested to successfully compile and run on that OS and it's targeted CPU architecture, while a question mark (?) shows that hsctl was not tested for that specific OS and CPU architecture combination, but may still work regardless.

hsctl should run on both ARM/ARM64 and x86-64 devices.

## Supported Headsets
As of now, hsctl supports the HyperX Cloud Stinger 2 Wireless.<br>

hsctl can:
- Get the battery level of the Stinger 2 Wireless,
- Get the charging status of the Stinger 2 Wireless,
- Get the microphone status of the Stinger 2 Wireless,
- Get and set the auto-shutdown time of the Stinger 2 Wireless

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

**Arch Linux**<br>
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

## Why?
While there are other projects similar to mine that are much more advanced and developed, hsctl is a hobby project. I have very little experience with programming, so I used this project as a way to learn how to program better in general (especially in Rust) and how to make open source projects like a developer in the industry. I am not a software engineer, though I would love to be one.

hsctl was only possible with the help of the hard work and dedication of other projects, most notably:
- [HyperHeadset](https://github.com/LennardKittner/HyperHeadset) by LennardKittner (helped with understanding Rust in the context of device I/O and using already existing code)
- [HyperX-Cloud-2-Battery-Monitor](https://github.com/auto94/HyperX-Cloud-2-Battery-Monitor) by auto94 (helped with understanding the protocol for the Stinger 2 Wireless)
- [hidapi-rs](https://github.com/ruabmbua/hidapi-rs) by ruabmbua (the Rust wrapper for the C [hidapi](https://github.com/libusb/hidapi) library)
- [egui](https://github.com/emilk/egui) by emilk (the GUI library used in hsctl)
- [crossbeam](https://github.com/crossbeam-rs/crossbeam) by crossbeam-rs (tools for concurrent programming in Rust)
- [tracing](https://github.com/tokio-rs/tracing) by tokio-rs (logging library for Rust)
- [clap](https://github.com/clap-rs/clap) by clap-rs (command line argument parser for Rust)
