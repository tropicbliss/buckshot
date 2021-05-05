mod cli;
mod config;
mod constants;
mod logic;
mod socket;

fn main() {
    cli::print_splash_screen();
    let config = config::Config::new();
    let sniper = logic::Sniper::new(config);
    sniper.snipe();
}
