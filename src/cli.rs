use anyhow::{bail, Result};
use dialoguer::Input;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author, about)]
pub struct Args {
    /// An optional argument for specifying the name you want to snipe
    #[structopt(short, long)]
    pub name: Option<String>,

    /// Name of config file (must be a TOML file)
    #[structopt(parse(from_os_str), short, long, default_value = "config.toml")]
    pub config_path: PathBuf,

    /// Indicate the number of invalid bearer tokens used to snipe names for testing
    #[structopt(short, long, name = "count")]
    pub test: Option<usize>,
}

impl Args {
    pub fn new() -> Result<Self> {
        let args = Self::from_args();
        if let Some(count) = args.test {
            if count == 0 {
                bail!("Test account count cannot be 0");
            }
        }
        Ok(args)
    }
}

pub fn get_name_choice() -> Result<String> {
    Ok(Input::new()
        .with_prompt("What name would you like to snipe")
        .interact()?)
}
