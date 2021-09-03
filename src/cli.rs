use anyhow::Result;
use dialoguer::Input;

pub fn get_name_choice() -> Result<String> {
    let name: String = Input::new()
        .with_prompt("What name would you like to snipe")
        .interact_text()?;
    Ok(name)
}
