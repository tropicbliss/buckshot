# buckshot

An asynchronous Minecraft username sniper written in Rust, powered by the Tokio runtime. It promises to be performant, capable, and easy to use.

## What it doesn't promise

I will try to be as honest as I can in this section as to why you shouldn't use this sniper (forgive my bluntness).

- Begineer-friendly (Setting up `config.toml` requires knowledge on TOML)
- Highly configurable (this sniper is highly opinionated and will do what it thinks is best, mostly because it is tailored to my own personal use)
- A thriving community (no Discord server and I don't feel like creating one, try [MCsniperGO](https://github.com/Kqzz/MCsniperGO) for an awesome community-driven sniper)
- Sniping with multiple accounts (I can't be bothered to add more features to this sniper)
- Adding more features in the future (everything including this sniper's website is a learning project. I don't care if you use it or not, I make no money off off it so improving it is a waste of my time)
- Lack of transparency when authenticating Microsoft accounts (porting over the code for authenticating a Microsoft account to this sniper is a chore, so I simply implemented an API to do this for me. For an end-user you have no idea what goes behind the scenes of my API, but I assure you I have zero interest in stealing your account's credentials. If you don't trust the process however, you probably should not use this sniper. And that is not without lack of effort. I tried to port over the [code](https://github.com/tropicbliss/xboxlive-auth) for authenticating Microsoft accounts. However, it does not work for all platforms and feels quite janky so I felt that it is not ready for production)

## Features (why you should use this sniper)

- Mojang account sniping
- Microsoft account sniping
- GC sniping
- Spread (delay between asynchronous sniping requests)
- Auto offset (this is actually a baller feature, every name I've sniped (600+ searches) uses auto offset, highly recommend)
- Change skin on snipe
- Name queueing
- Low latency by opening the TCP connections before sending requests
- High requests/second by bypassing HTTP overhead (thanks [arceus-sniper](https://github.com/aquild/arceus) for yoinking your feature list phrasing)

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
