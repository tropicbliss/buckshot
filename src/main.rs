mod cli;
mod config;
mod constants;
mod logic;
mod requests;
mod sockets;

use std::path::PathBuf;
use structopt::StructOpt;

/// A Minecraft name sniper made in Rust. Performant and capable.
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
async fn main() {
    let args = Args::new();
    cli::print_splash_screen();
    let config = config::Config::new(args.config_name).await;
    let snipe_task = impl_chooser(&config);
    let sniper = logic::Sniper::new(snipe_task, args.username_to_snipe, config, args.giftcode);
    sniper.run().await;
}

fn impl_chooser(config: &config::Config) -> logic::SnipeTask {
    type Task = logic::SnipeTask;
    if !config.config.microsoft_auth {
        if config.config.gc_snipe {
            bunt::println!(
                "{$red}`microsoft_auth` is set to false yet `gc_snipe` is set to true, defaulting to GC sniping instead.{/$}"
            );
            Task::Giftcode
        } else {
            Task::Mojang
        }
    } else if config.config.gc_snipe {
        Task::Giftcode
    } else {
        Task::Microsoft
    }
}
