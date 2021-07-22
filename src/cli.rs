use std::{io, process};

pub fn print_splash_screen() {
    fn get_logo() -> String {
        r#"______ _   _ _____  _   __ _____ _   _ _____ _____ 
| ___ \ | | /  __ \| | / //  ___| | | |  _  |_   _|
| |_/ / | | | /  \/| |/ / \ `--.| |_| | | | | | |  
| ___ \ | | | |    |    \  `--. \  _  | | | | | |  
| |_/ / |_| | \__/\| |\  \/\__/ / | | \ \_/ / | |  
\____/ \___/ \____/\_| \_/\____/\_| |_/\___/  \_/  
                                                   
                                                   "#
        .to_string()
    }
    fn get_credits() -> String {
        "Developed by @tropicbliss#0027 on Discord.".to_string()
    }
    bunt::println!("{$red}{}{/$}", get_logo());
    println!();
    bunt::println!("{$green}{}{/$}", get_credits());
    println!();
}

pub fn get_username_choice() -> String {
    loop {
        let mut input = String::new();
        print!("What name will you like to snipe: ");
        io::Write::flush(&mut io::stdout()).unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if !username_filter_predicate(input) {
            println!("Invalid username entered, please try again.");
            continue;
        } else {
            break input.to_string();
        }
    }
}

pub fn username_filter_predicate(username: &str) -> bool {
    username.len() > 2
        && username.len() < 17
        && username
            .chars()
            .all(|x| char::is_alphanumeric(x) || x == '_')
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
    pretty_panik(
        fid,
        &format!("HTTP status code: {}. Please try again later.", code),
    );
}

pub fn kalm_panik(fid: &str, err: &str) {
    bunt::eprintln!("{$red}Error{/$}: [{}] {}", fid, err);
}
