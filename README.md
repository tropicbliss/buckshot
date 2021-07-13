# buckshot

An asynchronous Minecraft username sniper written in Rust, powered by the Tokio runtime. It promises to be performant and capable.

A successor to NodeSniper, this sniper promises to be noob-friendly and straight-forward. If you are not wary of downloading random .exe files from the internet, operating this sniper is as simple as double-clicking the executable. If you are wary however, this readme also shows you how to compile this sniper on your own computer [here](https://github.com/chronicallyunfunny/buckshot#compiling-from-source) (from source code to an executable) and you can even run it off on relatively less supported hardware like a Raspberry Pi if you wish to do so.

No more new features are going to be added to this sniper. Further developments will be focused on maintaining this sniper and the authentication server.

## Features

- Mojang account sniping
- Microsoft account sniping
- GC sniping
- Spread (delay between asynchronous sniping requests)
- Auto offset (never rely on this feature for reliable sniping, it should only be used to gauge the delay for first time snipers, adjust upon that offset for subsequent snipes)
- Change skin on snipe
- Name queueing
- Low latency by opening the TCP connections before sending requests
- High requests/second by bypassing HTTP overhead (thanks arceus-sniper for yoinking your feature list phrasing)

## Credits ❤️

- Teun for the drop-time caching API

## Setup

1. Download the sniper [here](https://github.com/chronicallyunfunny/buckshot/releases/tag/v2.0.0).
2. Use Dimension 4 on Windows for time synchronisation.
3. For macOS and Linux users open up the terminal and navigate to the working directory and run `./buckshot`. You could also double-click the executable. I assume you use MCsniperPY so this process should be relatively straightforward.
4. This should generate `config.toml` on the same directory in which you can edit.
5. Read the errors. I've spent a large amount of time making the errors easy to read. If you encounter HTTP errors, something might be wrong with the internet or the servers on the other end.

## Moar tips for sniping

- Snipe with a VPS close to `api.minecraftservices.com` origin server (in N. Virginia) as it will minimise ping fluctuations.
- Even though this sniper is asynchronous and it is possible to snipe a name with one thread, using 2 threads will be the most ideal for non-GC snipes (and 6 threads for GC snipes).

## Command line arguments

Run `buckshot --help` or `./buckshot --help`.

## Compiling from source

1. Download the `rustup` toolchain right [here](https://rustup.rs/). Follow the instructions for your platform.
2. Run `git clone https://github.com/chronicallyunfunny/buckshot.git` in an appropriate directory to clone the repo.
3. In the folder named `buckshot`, run `cargo build --release`. The resulting executable file after compilation should be in the `target/release/` directory relative from the `buckshot` folder. If you encounter any errors throughout the compilation process, read through the errors as they generally tell you exactly what to do. Typically, when compiling for Linux, you'll need `libssl-dev`, `build-essential`, and `pkg-config`.

## Bug reporting

Feel free to use the GitHub issues tab. This is a new sniper so there may be tons of bugs.
