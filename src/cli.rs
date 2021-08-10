use ansi_term::Colour::{Green, Red};
use anyhow::{anyhow, Context, Result};
use std::io;
use std::io::{stdin, stdout, Write};

pub fn print_splash_screen() -> Result<()> {
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
        "Developed by @tropicbliss#0408 on Discord".to_string()
    }
    fn best_sniper() -> String {
        "THIS SNIPER IS 100% FREE ON GITHUB".to_string()
    }
    writeln!(stdout(), "{}", Red.paint(get_logo()))
        .with_context(|| anyhow!("Failed to print logo"))?;
    writeln!(stdout(), "{}", Green.paint(get_credits()))
        .with_context(|| anyhow!("Failed to print credits"))?;
    writeln!(stdout(), "{}", Green.paint(best_sniper()))
        .with_context(|| anyhow!("Failed to print best sniper prompt"))?;
    Ok(())
}

pub fn get_username_choice() -> Result<String> {
    Ok(loop {
        let mut input = String::new();
        write!(stdout(), "What name will you like to snipe: ")?;
        Write::flush(&mut io::stdout())?;
        stdin().read_line(&mut input)?;
        let input = input.trim();
        if username_filter_predicate(input) {
            break input.to_string();
        }
        writeln!(stdout(), "Invalid username entered, please try again")?;
        continue;
    })
}

pub fn username_filter_predicate(username: &str) -> bool {
    username.len() > 2
        && username.len() < 17
        && username
            .chars()
            .all(|x| char::is_alphanumeric(x) || x == '_')
}
