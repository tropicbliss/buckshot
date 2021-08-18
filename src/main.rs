#![warn(clippy::pedantic)]

mod cli;
mod config;
mod logic;
mod requests;
mod sockets;

use anyhow::{Context, Result};
use console::style;
use std::io::{stdout, Write};
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

type Task = logic::SnipeTask;

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
    let snipe_task = impl_chooser(&config).with_context(|| "Failed to choose sniping task")?;
    let mut sniper = logic::Sniper::new(snipe_task, args.username_to_snipe, config, args.giftcode)
        .with_context(|| "Failed to run sniper instance")?;
    sniper.run().await.with_context(|| "Failed to snipe name")?;
    Ok(())
}

fn impl_chooser(config: &config::Config) -> Result<Task> {
    let snipe_task = if !config.config.microsoft_auth {
        if config.config.gc_snipe {
            writeln!(stdout(), "{}", style("`microsoft_auth` is set to false yet `gc_snipe` is set to true, defaulting to GC sniping instead").red())?;
            Task::Giftcode
        } else {
            Task::Mojang
        }
    } else if config.config.gc_snipe {
        Task::Giftcode
    } else {
        Task::Microsoft
    };
    Ok(snipe_task)
}
