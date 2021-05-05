use std::io;

// Nice ASCII art inspired by Doom
pub fn print_splash_screen() {
    println!(r"______ _   _ _____  _   __ _____ _   _ _____ _____ ");
    println!(r"| ___ \ | | /  __ \| | / //  ___| | | |  _  |_   _|");
    println!(r"| |_/ / | | | /  \/| |/ / \ `--.| |_| | | | | | |  ");
    println!(r"| ___ \ | | | |    |    \  `--. \  _  | | | | | |  ");
    println!(r"| |_/ / |_| | \__/\| |\  \/\__/ / | | \ \_/ / | |  ");
    println!(r"\____/ \___/ \____/\_| \_/\____/\_| |_/\___/  \_/  ");
    println!("                                                   ");
    println!("                                                   ");
    println!("");
    println!("Developed by @chronicallyunfunny#1113 on Discord.");
    println!("");
}

// Get username choice for sniping, with low-level code for scanning user input
pub fn get_username_choice() -> String {
    let mut input;
    loop {
        input = String::new();
        print!("What name will you like to snipe: ");
        io::Write::flush(&mut io::stdout()).unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.len() < 3 || input.len() > 16 || !input.chars().all(|x| is_valid_username_char(x))
        {
            println!("Invalid username entered, please try again.");
            continue;
        } else {
            break;
        }
    }
    input
}

// Get offset similar to get_username_choice() except with string parsing to i32
pub fn get_offset() -> i32 {
    let mut input;
    let res;
    loop {
        input = String::new();
        print!("What will be your offset: ");
        io::Write::flush(&mut io::stdout()).unwrap();
        io::stdin().read_line(&mut input).unwrap();
        match input.lines().collect::<String>().parse::<i32>() {
            Ok(x) => {
                res = x;
                break;
            }
            Err(_) => {
                println!("Invalid offset entered, please try again.");
                continue;
            }
        }
    }
    res
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

pub fn get_access_token() -> String {
    let mut input = String::new();
    print!(
        r#"Sign in with your Microsoft account and copy the ID from the "access_token" field right here: "#
    );
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn get_refresh_token() -> String {
    let mut input = String::new();
    print!(r#"Copy the ID from the "refresh_token" field right here: "#);
    io::Write::flush(&mut io::stdout()).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// Used for closure for validating individual chars to determine if char in username is valid in iterable
fn is_valid_username_char(c: char) -> bool {
    if char::is_alphanumeric(c) {
        true
    } else {
        if c == '_' {
            true
        } else {
            false
        }
    }
}
