mod cli;
mod config;
mod constants;
mod requests;
mod runner;
mod sockets;
use argh::FromArgs;

#[derive(FromArgs)]
/// A Minecraft username sniper made in Rust. Performant and capable.
struct Options {
    /// an optional argument for specifying the username you want to snipe
    #[argh(option, short = 'u')]
    username_to_snipe: Option<String>,

    /// an optional argument for specifying the name of the config file (must be a TOML file)
    #[argh(option, short = 'c')]
    config_name: Option<String>,
}

#[tokio::main]
async fn main() {
    let (username_to_snipe, config_name) = get_envargs();
    cli::print_splash_screen();
    let config = config::Config::new(&config_name).await;
    let snipe_task = impl_chooser(&config);
    let sniper = runner::Sniper::new(snipe_task, username_to_snipe, config);
    sniper.run().await;
}

fn impl_chooser(config: &config::Config) -> runner::SnipeTask {
    type Task = runner::SnipeTask;
    if !config.config.microsoft_auth {
        if config.config.gc_snipe {
            println!(
                r#""microsoft_auth" is set to false yet "gc_snipe" is set to true. Defaulting to gift code sniping instead."#
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

fn get_envargs() -> (Option<String>, Option<String>) {
    let options: Options = argh::from_env();
    (options.username_to_snipe, options.config_name)
}
