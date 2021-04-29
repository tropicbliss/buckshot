mod cli;
mod constants;
mod logic;
mod requests;
mod socket;

fn main() {
    cli::print_splash_screen();
    let sniper = logic::Sniper::new();
    sniper.setup();
    let username = cli::get_username_choice();
    let offset = cli::get_offset();
    sniper.snipe(username, offset);
}
