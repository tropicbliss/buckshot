use crate::constants;
use anyhow::{bail, Result};
use serde::{de::Error, Deserialize, Deserializer};
use std::{convert::From, fs::read_to_string};

#[derive(Deserialize)]
struct PrivateConfig {
    account_entry: Vec<AccountVariants>,
    offset: u32,
    #[serde(deserialize_with = "to_task")]
    mode: SnipeTask,
    skin: Option<Skin>,
    name_queue: Option<NameQueue>,
}

#[derive(Deserialize)]
pub struct NameQueue {
    pub queue: Vec<String>,
    pub never_stop_sniping: bool,
}

#[derive(Deserialize)]
#[serde(from = "PrivateConfig")]
pub struct Config {
    pub account_entry: Vec<Account>,
    pub offset: u32,
    pub mode: SnipeTask,
    pub skin: Option<Skin>,
    pub name_queue: Option<NameQueue>,
}

#[derive(PartialEq)]
pub enum SnipeTask {
    Mojang,
    Microsoft,
    Giftcode,
}

#[derive(Deserialize)]
pub struct Skin {
    pub file: bool,
    pub path: String,
    pub slim: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AccountVariants {
    One(One),
    Two(Two),
    Three(Three),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct One {
    email: String,
    password: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Two {
    email: String,
    password: String,
    sq_ans: [String; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Three {
    bearer: String,
}

#[derive(Clone)]
pub struct Account {
    pub email: Option<String>,
    pub password: Option<String>,
    pub sq_ans: Option<[String; 3]>,
    pub bearer: Option<String>,
}

impl From<PrivateConfig> for Config {
    fn from(item: PrivateConfig) -> Self {
        let account_entry = item
            .account_entry
            .into_iter()
            .map(|entry| match entry {
                AccountVariants::One(x) => Account {
                    email: Some(x.email),
                    password: Some(x.password),
                    sq_ans: None,
                    bearer: None,
                },
                AccountVariants::Two(x) => Account {
                    email: Some(x.email),
                    password: Some(x.password),
                    sq_ans: Some(x.sq_ans),
                    bearer: None,
                },
                AccountVariants::Three(x) => Account {
                    email: None,
                    password: None,
                    sq_ans: None,
                    bearer: Some(x.bearer),
                },
            })
            .collect();
        Self {
            account_entry,
            offset: item.offset,
            mode: item.mode,
            skin: item.skin,
            name_queue: item.name_queue,
        }
    }
}

fn to_task<'de, D>(deserializer: D) -> Result<SnipeTask, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_ascii_lowercase().as_ref() {
        "mj" | "mja" => Ok(SnipeTask::Mojang),
        "ms" | "msa" => Ok(SnipeTask::Microsoft),
        "prename" | "msprename" | "msaprename" | "pr" => Ok(SnipeTask::Giftcode),
        _ => Err(Error::custom("Invalid value")),
    }
}

pub fn new() -> Result<Config> {
    let cfg = read_to_string(constants::CONFIG_PATH)?;
    let cfg: Config = toml::from_str(&cfg)?;
    if cfg.account_entry.is_empty() {
        bail!("No accounts provided in config file");
    }
    if let Some(count) = &cfg.name_queue {
        if count.queue.is_empty() {
            bail!("No name provided in name queue");
        }
    }
    Ok(cfg)
}
