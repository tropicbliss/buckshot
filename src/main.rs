#![warn(clippy::pedantic)]

mod cli;
mod config;
mod constants;
mod logic;
mod requests;
mod sockets;

use std::path::PathBuf;
use structopt::StructOpt;
use ansi_term::Colour::Red;
use anyhow::{Context, Result};
use std::io::{stdout, Write};

/// A Minecraft name sniper written in Rust. Performant and capable.
#[derive(StructOpt, Debug)]
#[structopt()]
struct Args {
    /// An optional argument for specifying the name you want to snipe
    #[structopt(short, long)]
    username_to_snipe: Option<String>,

    /// An optional argument for specifying the name of the config file (must be a TOML file)
    #[structopt(short, long, default_value = "config.toml")]
    config_name: PathBuf,

    /// An optional argument for specifying the giftcode if you want the sniper to redeem the giftcode for you
    #[structopt(short, long)]
    giftcode: Option<String>,
}

impl Args {
    pub fn new() -> Self {
        Self::from_args()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::new();
    cli::print_splash_screen().with_context(|| "Failed to print splash screen")?;
    let config = config::Config::new(args.config_name).await.with_context(|| "Failed to get config options")?;
    let snipe_task = impl_chooser(&config).with_context(|| "Failed to choose implementation")?;
    let sniper = logic::Sniper::new(snipe_task, args.username_to_snipe, config, args.giftcode);
    sniper.run().await.with_context(|| "Failed to snipe name")?;
    Ok(())
}

fn impl_chooser(config: &config::Config) -> Result<logic::SnipeTask> {
    type Task = logic::SnipeTask;
    let paradigm = if !config.config.microsoft_auth {
        if config.config.gc_snipe {
            writeln!(stdout(), "{}", Red.paint("`microsoft_auth` is set to false yet `gc_snipe` is set to true, defaulting to GC sniping"))?;
            Task::Giftcode
        } else {
            Task::Mojang
        }
    } else if config.config.gc_snipe {
        Task::Giftcode
    } else {
        Task::Microsoft
    };
    Ok(paradigm)
}
