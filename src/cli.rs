use anyhow::Result;
use dialoguer::Input;
use std::io::{stdout, Write};

pub fn get_username_choice() -> Result<String> {
    Ok(loop {
        let input: String = Input::new()
            .with_prompt("What name would you like to snipe")
            .interact_text()?;
        if username_filter_predicate(&input) {
            break input;
        }
        writeln!(stdout(), "Invalid username entered, please try again")?;
    })
}

pub fn username_filter_predicate(username: &str) -> bool {
    username.len() >= 3
        && username.len() <= 16
        && username
            .chars()
            .all(|x| char::is_alphanumeric(x) || x == '_')
}
