use anyhow::Result;
use dialoguer::Input;
use std::io::{stdout, Write};

pub fn get_name_choice() -> Result<String> {
    Ok(loop {
        let input: String = Input::new()
            .with_prompt("What name would you like to snipe")
            .interact_text()?;
        if name_filter_predicate(&input) {
            break input;
        }
        writeln!(stdout(), "Invalid name entered, please try again")?;
    })
}

pub fn name_filter_predicate(name: &str) -> bool {
    name.len() >= 3
        && name.len() <= 16
        && name.chars().all(|x| char::is_alphanumeric(x) || x == '_')
}
