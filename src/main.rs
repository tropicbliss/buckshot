mod cli;
mod config;
mod constants;
mod logic;
use std::env;

fn main() {
    cli::print_splash_screen();
    let args: Vec<String> = env::args().collect();
    let arg_username = if args.len() > 0 {
        Some(args[1].clone())
    } else {
        None
    };
    let config = config::Config::new();
    let sniper = logic::Sniper::new(config, arg_username);
    sniper.snipe();
}
