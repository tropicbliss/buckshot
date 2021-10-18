# buckshot

A fast and capable asynchronous Minecraft name sniper.

## Features

- Mojang account sniping
- Microsoft account sniping
- GC sniping
- Spread (delay between asynchronous sniping requests)
- Auto offset
- Change skin on snipe
- Name queueing
- Multi account support for GC sniping

## Issues

- Microsoft authentication does not work on macOS

## Usage

1. Download the [latest release](https://github.com/tropicbliss/buckshot/releases/latest) for your operating system. Alternatively, you can [compile from source](https://github.com/tropicbliss/buckshot#compiling-from-source).
2. Create `config.toml`. Add your accounts and enter your offset. Refer to this [guide](https://github.com/tropicbliss/buckshot/blob/main/CONFIG.md) for more information.
3. Run the sniper.
  - Open a terminal ([instructions for macOS users](https://www.stugon.com/open-terminal-in-current-folder-location-mac/)) (if you're on Windows, do yourself a favor and use Windows Terminal instead of `cmd.exe`) and use `cd` to navigate to the folder where the binary is located.
  - For non-Windows users, you will also have to run `sudo chmod +x binary_name_here`.
  - Run `./binary_name_here`.
4. The sniper might prompt you for a username (depending on your configuration). If it does, enter that and then the sniper will authenticate (this is run 8 hours before snipe) and then count down.

## Command line arguments

Run `./buckshot --help`.

## Compiling from source

1. Download the `rustup` toolchain right [here](https://rustup.rs/). Follow the instructions for your platform.
2. Run `cargo install --git https://github.com/tropicbliss/buckshot.git`. If you encounter any errors throughout the compilation process, read through the errors as they generally tell you exactly what to do. Typically, when compiling for Linux, you'll need `build-essential`, `pkg-config`, and `libssl-dev`.
3. You should be able to just run `buckshot` from anywhere on your system.
