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
        if input.len() < 3
            || input.len() > 16
            || !input.chars().all(|x| char::is_alphanumeric(x) || x == '_')
        {
            println!("Invalid username entered, please try again.");
            continue;
        } else {
            break input;
        }
    }
}

pub fn get_giftcode() -> Option<String> {
    let mut input = String::new();
    print!("Enter your gift code (press ENTER if you have already redeemed your gift code): ");
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    if input.is_empty() {
        None
    } else {
        Some(input)
    }
}

pub fn get_access_token() -> String {
    let mut input = String::new();
    print!(
        r#"Sign in with your Microsoft account and copy the access token from the authentication page right here: "#
    );
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    input
}

pub fn exit_program() {
    let mut input = String::new();
    print!("Press ENTER to quit: ");
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

pub fn pretty_panic(err: &str) -> ! {
    bunt::eprintln!("{$red}Error{/$}: {}", err);
    exit_program();
    process::exit(1);
}

pub fn kalm_panic(err: &str) {
    bunt::eprintln!("{$red}Error{/$}: {}", err);
}
