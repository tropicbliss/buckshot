# buckshot
A Minecraft username sniper made in Rust. Performant and capable.

A successor to NodeSniper, this sniper promises to be noob-friendly and straight-forward. If you are not wary of downloading random .exe files from the internet, operating this sniper is as simple as double-clicking the executable. If you are wary however, this readme also shows you how to compile this sniper on your own computer [here](https://github.com/chronicallyunfunny/buckshot#compiling-from-source) (from source code to an executable) and you can even run it off on relatively less supported hardware like a Raspberry Pi if you wish to do so.

## For sniper developers

Microsoft authentication server link this sniper uses:

https://login.live.com/oauth20_authorize.srf?client_id=68f2f45b-02e2-4625-8225-25c6fcc25039&response_type=code&redirect_uri=https://buckshotrs.com/auth:1338&scope=XboxLive.signin%20offline_access&state=STORAGE_ID

## Features

- Mojang account sniping
- Microsoft account sniping
- GC sniping
- Spread (delay between asynchronous sniping requests)
- Auto offset (never rely on this feature for reliable sniping, it should only be used to gauge the delay for first time snipers, adjust upon that delay for subsequent snipes)
- Change skin on snipe

## Credits ❤️

- Teun for the drop-time caching API

## Setup

1. Download the sniper [here](https://github.com/chronicallyunfunny/buckshot/releases/tag/v0.4.0).
2. Use Dimension 4 on Windows for time synchronisation.
3. Open up `config.toml` on your favourite editor and configure it if need be.
4. Make sure `config.toml` is on the same directory as the executable and double click it. For macOS and Linux users open up the terminal and navigate to the working directory and run `./buckshot`. I assume you use MCsniperPY so this process should be relatively straightforward.
5. Read the errors. I've spent a large amount of time making the errors easy to read. If you encounter HTTP errors, something might be wrong with the internet or the servers on the other end.

## Moar tips for sniping

- Snipe with a VPS close to `api.minecraftservices.com` origin server (in N. Virginia) as it will minimise ping fluctuations.
- Even though this sniper is asynchronous and it is possible to snipe a name with one thread, using 2 threads will be the most ideal for non-GC snipes (and 6 threads for GC snipes).

## Command line arguments

Run `buckshot --help` or `./buckshot --help`.

## Compiling from source

1. Download the `rustup` toolchain right [here](https://rustup.rs/). Follow the instructions for your platform.
2. Run `git clone https://github.com/chronicallyunfunny/buckshot.git` in an appropriate directory to clone the repo.
3. In the folder named `buckshot`, run `cargo build --release`. The resulting executable file after compilation should be in the `target/release/` directory relative from the `buckshot` folder. If you encounter any errors throughout the compilation process, read through the errors as they generally tell you exactly what to do. Typically, when compiling for Linux, you'll need dev packages for OpenSSL and a package to trace the OpenSSL directory.

## Bug reporting

Feel free to use the GitHub issues tab. This is a new sniper so there may be tons of bugs.
