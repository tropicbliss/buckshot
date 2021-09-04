use anyhow::Result;
use dialoguer::Input;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub struct Args {
    /// An optional argument for specifying the name you want to snipe
    #[structopt(short, long)]
    pub name: Option<String>,

    /// Name of config file (must be a TOML file)
    #[structopt(parse(from_os_str), short, long, default_value = "config.toml")]
    pub config_path: PathBuf,
}

impl Args {
    pub fn new() -> Self {
        Self::from_args()
    }
}

pub fn get_name_choice() -> Result<String> {
    let name: String = Input::new()
        .with_prompt("What name would you like to snipe")
        .interact_text()?;
    Ok(name)
}
