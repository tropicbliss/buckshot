#![warn(clippy::pedantic)]

mod cli;
mod config;
mod constants;
mod msauth;
mod requests;
mod sockets;

use anyhow::{bail, Context, Result};
use chrono::{Duration, Local, TimeZone};
use console::style;
use std::{
    io::{stdout, Write},
    thread::sleep,
};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    type SnipeTask = config::SnipeTask;
    let args = cli::Args::new();
    let mut config =
        config::new().with_context(|| format!("Failed to get config options from {}", constants::CONFIG_PATH))?;
    let task = &config.mode;
    if config.name_queue.is_none() || !config.name_queue.clone().unwrap().never_stop_sniping {
        if task != &SnipeTask::Giftcode && config.account_entry.len() > 1 {
            bail!("Unable to use more than one normal account");
        } else if config.account_entry.len() > 10 {
            bail!("Unable to use more than 10 prename accounts");
        }
    }
    let name_list = if let Some(name) = args.name {
        vec![name]
    } else if let Some(x) = &config.name_queue {
        x.queue.clone()
    } else {
        let name = cli::get_name_choice().with_context(|| "Failed to get name choice")?;
        vec![name]
    };
    let requestor = requests::Requests::new()?;
    for (count, name) in name_list.into_iter().enumerate() {
        let name = name.trim();
        if count != 0 {
            writeln!(stdout(), "Moving on to next name...")?;
            writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?;
            sleep(std::time::Duration::from_secs(20));
        }
        writeln!(stdout(), "Initialising...")?;
        let droptime = if let Some(timestamp) = args.timestamp {
            Local.timestamp(timestamp, 0)
        } else {
            match requestor.check_name_availability_time(name).with_context(|| format!("Failed to get the droptime of {}", name))? {
                requests::DroptimeData::Available(droptime) => droptime,
                requests::DroptimeData::Unavailable(error) => {
                    writeln!(
                        stdout(),
                        "{}",
                        style(format!("Failed to get the droptime of {}: {}", name, error)).red()
                    )?;
                    continue;
                }
            }
        };
        let formatted_droptime = droptime.format("%F %T");
        writeln!(
            stdout(),
            "Sniping {} at {} with an offset of {} ms",
            name,
            formatted_droptime,
            config.offset
        )?;
        let snipe_time = droptime - Duration::milliseconds(i64::from(config.offset));
        let setup_time = snipe_time - Duration::hours(12);
        if Local::now() < setup_time {
            let sleep_duration = (setup_time - Local::now())
                .to_std()
                .unwrap_or(std::time::Duration::ZERO);
            sleep(sleep_duration);
            if args.timestamp.is_none() {
                if let requests::DroptimeData::Unavailable(error) = requestor
                    .check_name_availability_time(name)
                    .with_context(|| format!("Failed to get the droptime of {}", name))?
                {
                    writeln!(
                        stdout(),
                        "{}",
                        style(format!("Failed to get the droptime of {}: {}", name, error)).red()
                    )?;
                    continue;
                }
            }
        }
        let mut bearer_tokens = Vec::new();
        let mut account_idx = 0;
        for (count, account) in config.account_entry.clone().iter().enumerate() {
            let bearer_token = if let Some(bearer) = account.bearer.clone() {
                bearer
            } else {
                if count != 0 {
                    writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?;
                    sleep(std::time::Duration::from_secs(20));
                }
                let (email, password) = (
                    account.email.as_ref().unwrap(),
                    account.password.as_ref().unwrap(),
                );
                let bearer = if task == &SnipeTask::Mojang {
                    match requestor
                        .authenticate_mojang(email, password, &account.sq_ans)
                        .with_context(|| {
                            format!("Failed to authenticate the Mojang account {}", email)
                        }) {
                        Ok(x) => x,
                        Err(y) => {
                            if config.account_entry.len() == 1 {
                                bail!(y);
                            }
                            writeln!(stdout(), "{}", style("Failed to authenticate a Mojang account, moving on to next account...").red())?;
                            config.account_entry.remove(account_idx);
                            continue;
                        }
                    }
                } else {
                    let authenticator = msauth::Auth::new(email, password)
                        .with_context(|| "Error creating Microsoft authenticator")?;
                    match authenticator.authenticate().with_context(|| {
                        format!("Failed to authenticate the Microsoft account {}", email)
                    }) {
                        Ok(x) => x,
                        Err(y) => {
                            if config.account_entry.len() == 1 {
                                bail!(y);
                            }
                            writeln!(stdout(), "{}", style("Failed to authenticate a Microsoft account, moving on to next account...").red())?;
                            config.account_entry.remove(account_idx);
                            continue;
                        }
                    }
                };
                if task != &SnipeTask::Giftcode {
                    if let Err(y) = requestor
                        .check_name_change_eligibility(&bearer)
                        .with_context(|| {
                            format!("Failed to check name change eligibility of {}", email)
                        })
                    {
                        if config.account_entry.len() == 1 {
                            bail!(y);
                        }
                        writeln!(
                            stdout(),
                            "{}",
                            style(format!(
                                "Failed to check name change eligibility of {}",
                                email
                            ))
                        )?;
                        config.account_entry.remove(account_idx);
                        continue;
                    }
                }
                bearer
            };
            bearer_tokens.push(bearer_token);
            if task != &SnipeTask::Giftcode
                || (task == &SnipeTask::Giftcode && bearer_tokens.len() == 10)
            {
                break;
            }
            account_idx += 1;
        }
        if bearer_tokens.is_empty() {
            bail!("No Microsoft accounts left to use");
        }
        writeln!(stdout(), "{}", style("Successfully signed in").green())?;
        writeln!(stdout(), "Setup complete")?;
        let mut is_success = None;
        let is_gc = task == &SnipeTask::Giftcode;
        let res_data = sockets::snipe_executor(name, &bearer_tokens, snipe_time, is_gc)
            .await
            .with_context(|| format!("Failed to execute the snipe of {}", name))?;
        for res in res_data {
            let formatted_timestamp = res.timestamp.format("%F %T%.6f");
            match res.status {
                200 => {
                    writeln!(
                        stdout(),
                        "[{}] {} @ {}",
                        style("success").green(),
                        style("200").green(),
                        style(format!("{}", formatted_timestamp)).cyan()
                    )?;
                    is_success = Some(res.account_idx);
                }
                status => {
                    writeln!(
                        stdout(),
                        "[{}] {} @ {}",
                        style("fail").red(),
                        style(format!("{}", status)).red(),
                        style(format!("{}", formatted_timestamp)).cyan()
                    )?;
                }
            }
        }
        if let Some(account_idx) = is_success {
            writeln!(
                stdout(),
                "{}",
                style(format!("Successfully sniped {}!", name)).green()
            )?;
            if let Some(skin) = &config.skin {
                let skin_model = if skin.slim { "slim" } else { "classic" }.to_string();
                requestor
                    .upload_skin(
                        &bearer_tokens[account_idx],
                        &skin.path,
                        skin_model,
                        skin.file,
                    )
                    .with_context(|| {
                        format!(
                            "Failed to change the skin of {}",
                            config.account_entry[account_idx].email.as_ref().unwrap()
                        )
                    })?;
                writeln!(stdout(), "{}", style("Successfully changed skin").green())?;
            }
            config.account_entry.remove(account_idx);
            if let Some(name_queue) = &config.name_queue {
                if name_queue.never_stop_sniping && !config.account_entry.is_empty() {
                    continue;
                }
            }
            break;
        }
        writeln!(stdout(), "Failed to snipe {}", name)?;
    }
    Ok(())
}
