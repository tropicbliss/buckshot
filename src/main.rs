#![warn(clippy::pedantic)]

mod cli;
mod config;
mod logic;
mod requests;
mod sockets;

use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

/// A performant and capable Minecraft name sniper
#[derive(StructOpt, Debug)]
#[structopt()]
struct Args {
    /// An optional argument for specifying the name you want to snipe
    #[structopt(short, long)]
    username_to_snipe: Option<String>,

    /// Name of config file (must be a TOML file)
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
    let config =
        config::Config::new(&args.config_name).with_context(|| "Failed to get config options")?;
    logic::run(args.username_to_snipe, config, args.giftcode)
        .await
        .with_context(|| "Failed to snipe name")?;
    Ok(())
}
