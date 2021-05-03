mod cli;
mod config;
mod constants;
mod logic;
mod socket;

fn main() {
    cli::print_splash_screen();
    let config = config::Config::new();
    let setup = logic::Setup::new(config.clone());
    setup.setup();
    let username = cli::get_username_choice();
    let offset;
    if config.config.auto_offset {
        offset = logic::auto_offset_calculation();
    } else {
        offset = cli::get_offset();
    }
    let sniper = logic::Sniper::new(setup, username, offset);
    sniper.snipe();
}
