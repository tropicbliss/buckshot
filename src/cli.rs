use anyhow::Result;
use dialoguer::Input;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(author, about)]
pub struct Args {
    /// Name to snipe
    #[structopt(short, long)]
    pub name: Option<String>,

    /// UNIX timestamp of name droptime
    #[structopt(short, long, requires = "name")]
    pub timestamp: Option<i64>,
}

impl Args {
    pub fn new() -> Self {
        Self::from_args()
    }
}

pub fn get_name_choice() -> Result<String> {
    Ok(Input::new()
        .with_prompt("What name would you like to snipe")
        .interact()?)
}
