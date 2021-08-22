# buckshot

An asynchronous Minecraft username sniper written in Rust, powered by the Tokio runtime. It promises to be performant, capable, and easy to use.

## What it doesn't promise

- Begineer-friendly (Setting up `config.toml` requires knowledge on TOML)
- Highly configurable (this sniper is highly opinionated and will do what it thinks is best)
- A thriving community (no Discord server and I don't feel like creating one)
- Sniping with multiple accounts (I really can't find a way to implement this in a TOML file that I feel 100% satisfied with)

## Features

- Mojang account sniping
- Microsoft account sniping
- GC sniping
- Spread (delay between asynchronous sniping requests)
- Auto offset (this is actually a baller feature, every name I've sniped (600+ searches) uses auto offset, highly recommend)
- Change skin on snipe
- Name queueing
- Low latency by opening the TCP connections before sending requests
- High requests/second by bypassing HTTP overhead (thanks arceus-sniper for yoinking your feature list phrasing)

## Setup

1. Download the sniper [here](https://github.com/chronicallyunfunny/buckshot/releases/latest).
2. Use Dimension 4 on Windows for time synchronisation.
3. Open up the terminal and navigate to the working directory and run `./buckshot-<platform>-<architecture>` (if you're on Windows, do yourself a favor and use Windows Terminal instead of `cmd.exe`). I assume you use MCsniperPY so this process should be relatively straightforward.
4. This should generate `config.toml` on the same directory in which you can edit.
5. Read the errors. I've spent a large amount of time making the errors easy to read. If you encounter HTTP errors, something might be wrong with the internet or the servers on the other end.

## Moar tips for sniping

- Snipe with a VPS close to `api.minecraftservices.com` origin server (in N. Virginia) as it will minimise ping fluctuations.
- Even though this sniper is asynchronous and it is possible to snipe a name with one thread, using a multi-threaded processor can improve performance.

## Command line arguments

Run `buckshot --help` or `./buckshot --help`.

## Compiling from source

1. Download the `rustup` toolchain right [here](https://rustup.rs/). Follow the instructions for your platform.
2. Run `git clone https://github.com/tropicbliss/buckshot.git` in an appropriate directory to clone the repo.
3. In the folder named `buckshot`, run `cargo build --release`. The resulting executable file after compilation should be in the `target/release/` directory, relative from the `buckshot` folder. If you encounter any errors throughout the compilation process, read through the errors as they generally tell you exactly what to do. Typically, when compiling for Linux, you'll need `build-essential`.

## Bug reporting

Feel free to use the GitHub issues tab. This is a new sniper so there may be tons of bugs.
