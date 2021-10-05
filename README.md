# buckshot

An asynchronous Minecraft username sniper written in Rust, powered by the Tokio runtime. It promises to be performant, capable, and easy to use.

## Features

- Mojang account sniping
- Microsoft account sniping
- GC sniping
- Spread (delay between asynchronous sniping requests)
- Auto offset
- Change skin on snipe
- Name queueing
- Multi account support for GC sniping
- Low latency by opening the TCP connections before sending requests
- High requests/second by bypassing HTTP overhead (thanks [arceus-sniper](https://github.com/aquild/arceus) for yoinking your feature list phrasing)

## Issues

- Microsoft authentication does not work on macOS

## Setup

1. Download the [latest release](https://github.com/chronicallyunfunny/buckshot/releases/latest) for your operating system.
2. Download Dimension 4 on Windows or `chrony` on Linux for accurate time synchronisation.
3. Open up the terminal and navigate to the directory where the sniper is downloaded to. If you are not on Windows, run `sudo chmod +x binary_name_here`.
4. Create a new config file. Click [here](https://github.com/tropicbliss/buckshot/blob/main/CONFIG.md) for instructions.
5. Run `./buckshot` (if you're on Windows, do yourself a favor and use Windows Terminal instead of `cmd.exe`). `config.toml` will appear in the current directory.
6. Run the sniper again with the same command used in step 4.

## Moar tips for sniping

- Snipe with a VPS close to `api.minecraftservices.com` origin server (in N. Virginia) as it will minimise ping fluctuations.
- Even though this sniper is asynchronous and it is possible to snipe a name with one thread, using a multi-threaded processor can improve performance.

## Command line arguments

Run `buckshot --help` or `./buckshot --help`.

## Compiling from source

1. Download the `rustup` toolchain right [here](https://rustup.rs/). Follow the instructions for your platform.
2. Run `cargo install --git https://github.com/tropicbliss/buckshot.git`. If you encounter any errors throughout the compilation process, read through the errors as they generally tell you exactly what to do. Typically, when compiling for Linux, you'll need `build-essential`, `pkg-config`, and `libssl-dev`.
3. You should be able to just run `buckshot` from anywhere on your system.

## Bug reporting

Feel free to use the GitHub issues tab. This is a new sniper so there may be tons of bugs.
