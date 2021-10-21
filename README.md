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

- Microsoft authentication does not work on macOS (manually specify bearer tokens instead)

## Usage

1. Download the [latest release](https://github.com/tropicbliss/buckshot/releases/latest) for your operating system.
2. Create `config.toml`. Add your accounts and enter your offset. Refer to this [guide](https://github.com/tropicbliss/buckshot/blob/main/CONFIG.md) for more information.
3. Run the sniper.
   - Open a terminal ([instructions for macOS users](https://www.stugon.com/open-terminal-in-current-folder-location-mac/)) (if you're on Windows, do yourself a favor and use Windows Terminal instead of `cmd.exe`) and use `cd` to navigate to the folder where the binary is located.
   - For non-Windows users, you will also have to run `sudo chmod +x binary_name_here`.
   - Run `./binary_name_here`.
4. The sniper might prompt you for a username (depending on your configuration). If it does, enter that and then the sniper will authenticate (this is run 8 hours before snipe) and then count down.

## Command line arguments

Run `./buckshot --help`.

## Compiling from source

If you are on another platform, compile the server yourself to try it out:

```sh
git clone https://github.com/tropicbliss/buckshot
cd buckshot
cargo build --release
```

Compiling from source requires the latest stable version of Rust. Older Rust versions may be able to compile `buckshot`, but they are not guaranteed to keep working.

The server executable will be located in `target/release`.
