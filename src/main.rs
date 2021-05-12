mod cli;
mod config;
mod constants;
mod requests;
mod runner;

#[tokio::main]
async fn main() {
    cli::print_splash_screen();
    let config = config::Config::new();
    let snipe_task = impl_chooser(&config);
    let username_to_snipe = get_username_arg();
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
    } else {
        if config.config.gc_snipe {
            Task::Giftcode
        } else {
            Task::Microsoft
        }
    }
}

fn get_username_arg() -> Option<String> {
    use std::env;
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        None
    } else {
        Some(args[1].clone())
    }
}
