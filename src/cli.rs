use std::{io, process};

pub fn print_splash_screen() {
    bunt::println!(r"{$red}______ _   _ _____  _   __ _____ _   _ _____ _____ {/$}");
    bunt::println!(r"{$red}| ___ \ | | /  __ \| | / //  ___| | | |  _  |_   _|{/$}");
    bunt::println!(r"{$red}| |_/ / | | | /  \/| |/ / \ `--.| |_| | | | | | |  {/$}");
    bunt::println!(r"{$red}| ___ \ | | | |    |    \  `--. \  _  | | | | | |  {/$}");
    bunt::println!(r"{$red}| |_/ / |_| | \__/\| |\  \/\__/ / | | \ \_/ / | |  {/$}");
    bunt::println!(r"{$red}\____/ \___/ \____/\_| \_/\____/\_| |_/\___/  \_/  {/$}");
    bunt::println!("                                                   ");
    bunt::println!("                                                   ");
    bunt::println!("");
    bunt::println!("{$green}Developed by @tropicbliss#0027 on Discord.{/$}");
    bunt::println!("");
}

pub fn get_username_choice() -> String {
    loop {
        let mut input = String::new();
        print!("What name will you like to snipe: ");
        io::Write::flush(&mut io::stdout()).unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.len() < 3
            || input.len() > 16
            || !input.chars().all(|x| char::is_alphanumeric(x) || x == '_')
        {
            println!("Invalid username entered, please try again.");
            continue;
        } else {
            break input.to_string();
        }
    }
}

pub fn get_giftcode() -> Option<String> {
    let mut input = String::new();
    print!("Enter your gift code (press ENTER if you have already redeemed your gift code): ");
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    if input.is_empty() {
        None
    } else {
        Some(input.to_string())
    }
}

pub fn exit_program() {
    let mut input = String::new();
    print!("Press ENTER to quit: ");
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

pub fn pretty_panik(fid: &str, err: &str) -> ! {
    bunt::eprintln!("{$red}Error{/$}: [{}] {}", fid, err);
    exit_program();
    process::exit(1);
}

pub fn http_timeout_panik(fid: &str) -> ! {
    pretty_panik(fid, "HTTP request timeout.");
}

pub fn http_not_ok_panik(fid: &str, code: u16) -> ! {
    pretty_panik(fid, &format!("HTTP status code: {}.", code));
}

pub fn kalm_panik(fid: &str, err: &str) {
    bunt::eprintln!("{$red}Error{/$}: [{}] {}", fid, err);
}
