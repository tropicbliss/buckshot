mod cli;
mod config;
mod msauth;
mod requests;
mod sockets;

use anyhow::{bail, Context, Result};
use chrono::{Duration, Utc};
use chrono_humanize::HumanTime;
use console::style;
use std::{
    io::{stdout, Write},
    thread::sleep,
};

#[derive(PartialEq)]
pub enum SnipeTask {
    Mojang,
    Microsoft,
    Giftcode,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::new();
    let mut config = config::Config::new(&args.config_path)
        .with_context(|| format!("Failed to parse {}", args.config_path.display()))?;
    let task = if !config.microsoft_auth {
        if config.gc_snipe {
            writeln!(stdout(), "{}", style("`microsoft_auth` is set to false yet `gc_snipe` is set to true, defaulting to GC sniping instead").red())?;
            SnipeTask::Giftcode
        } else {
            SnipeTask::Mojang
        }
    } else if config.gc_snipe {
        SnipeTask::Giftcode
    } else {
        SnipeTask::Microsoft
    };
    if task != SnipeTask::Giftcode && config.account_entry.len() > 1 {
        bail!(
            "You can only provide 1 account in config file as sniper is not set to GC sniping mode"
        );
    }
    if config.account_entry.is_empty() {
        bail!("No accounts provided in config file");
    }
    if config.account_entry.len() > 10 {
        bail!("Only a max of 10 accounts is allowed when GC sniping");
    }
    let name_list = if let Some(name) = args.name {
        vec![name]
    } else if let Some(x) = config.name_queue.clone() {
        x
    } else {
        let name = cli::get_name_choice().with_context(|| "Failed to get name choice")?;
        vec![name]
    };
    if name_list.is_empty() {
        bail!("No name provided in name queue");
    }
    let requestor = requests::Requests::new()?;
    for (count, name) in name_list.into_iter().enumerate() {
        if count != 0 {
            writeln!(stdout(), "Moving on to next name...")?;
            writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?;
            sleep(std::time::Duration::from_secs(20));
        }
        writeln!(stdout(), "Initialising...")?;
        let droptime = match requestor
            .check_name_availability_time(&name)
            .with_context(|| format!("Failed to get the droptime of {}", name))?
        {
            requests::DroptimeData::Available(droptime) => droptime,
            requests::DroptimeData::Unavailable(error) => {
                writeln!(
                    stdout(),
                    "{}",
                    style(format!("Failed to get droptime of {}: {}", name, error)).red()
                )?;
                continue;
            }
        };
        let is_gc = task == SnipeTask::Giftcode;
        let executor = sockets::Executor::new(&name, is_gc);
        let offset = if let Some(x) = config.offset {
            x
        } else {
            writeln!(stdout(), "Calculating offset...")?;
            executor
                .auto_offset_calculator()
                .await
                .with_context(|| "Failed to calculate offset")?
        };
        writeln!(stdout(), "Your offset is: {} ms", offset)?;
        let formatted_droptime = droptime.format("%F %T");
        let wait_time = droptime - Utc::now();
        let formatted_wait_time = HumanTime::from(wait_time);
        writeln!(
            stdout(),
            r#"Sniping "{}" {} | sniping at {} (utc)"#,
            name,
            formatted_wait_time,
            formatted_droptime
        )?;
        let snipe_time = droptime - Duration::milliseconds(offset);
        let setup_time = snipe_time - Duration::hours(12);
        if Utc::now() < setup_time {
            let sleep_duration = match (setup_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => std::time::Duration::ZERO,
            };
            sleep(sleep_duration);
            if let requests::DroptimeData::Unavailable(error) = requestor
                .check_name_availability_time(&name)
                .with_context(|| format!("Failed to get the droptime of {}", name))?
            {
                writeln!(
                    stdout(),
                    "{}",
                    style(format!("Failed to get droptime of {}: {}", name, error)).red()
                )?;
                continue;
            }
        }
        let mut bearer_tokens = Vec::with_capacity(config.account_entry.len());
        let mut is_warned = false;
        let mut account_idx = 0;
        for (count, account) in config.account_entry.clone().iter().enumerate() {
            if count != 0 {
                writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?;
                sleep(std::time::Duration::from_secs(20));
            }
            let bearer_token = if task == SnipeTask::Mojang {
                requestor
                    .authenticate_mojang(&account.email, &account.password, &account.sq_ans)
                    .with_context(|| {
                        format!(
                            "Failed to authenticate the Mojang account: {}",
                            account.email
                        )
                    })?
            } else {
                let authenticator = msauth::Auth::new(&account.email, &account.password)
                    .with_context(|| "Error creating Microsoft authenticator")?;
                match authenticator.authenticate().with_context(|| {
                    format!(
                        "Failed to authenticate the Microsoft account: {}",
                        account.email
                    )
                }) {
                    Ok(x) => x,
                    Err(y) => {
                        if config.account_entry.len() == 1 {
                            bail!(y)
                        }
                        writeln!(stdout(), "{}", style("Failed to authenticate a Microsoft account, removing it from the list...").red())?;
                        config.account_entry.remove(account_idx);
                        continue;
                    }
                }
            };
            if task == SnipeTask::Giftcode && count == 0 {
                if let Some(gc) = &account.giftcode {
                    match requestor
                        .redeem_giftcode(&bearer_token, gc)
                        .with_context(|| {
                            format!(
                                "Failed to redeem the giftcode of the account: {}",
                                account.email
                            )
                        }) {
                        Ok(_) => {
                            writeln!(
                                stdout(),
                                "{}",
                                style("Successfully redeemed giftcode").green()
                            )?;
                        }
                        Err(y) => {
                            if config.account_entry.len() == 1 {
                                bail!(y);
                            }
                            writeln!(stdout(), "{}", style("Failed to redeem the giftcode of an account, removing it from the list...").red())?;
                            config.account_entry.remove(account_idx);
                            continue;
                        }
                    }
                } else if !is_warned {
                    writeln!(
                        stdout(),
                        "{}",
                        style("Reminder: You should redeem your giftcode before GC sniping").red()
                    )?;
                    is_warned = true;
                }
            }
            account_idx += 1;
            if task != SnipeTask::Giftcode {
                requestor
                    .check_name_change_eligibility(&bearer_token)
                    .with_context(|| {
                        format!(
                            "Failed to check name change eligibility of {}",
                            account.email
                        )
                    })?;
            }
            bearer_tokens.push(bearer_token);
        }
        if bearer_tokens.is_empty() {
            bail!("No Microsoft accounts left to use");
        }
        writeln!(stdout(), "{}", style("Successfully signed in").green())?;
        writeln!(stdout(), "Setup complete")?;
        let mut is_success = None;
        let res_data = executor
            .snipe_executor(&bearer_tokens, config.spread, snipe_time)
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
                        skin.is_file,
                    )
                    .with_context(|| {
                        format!(
                            "Failed to change the skin of {}",
                            config.account_entry[account_idx].email
                        )
                    })?;
                writeln!(stdout(), "{}", style("Successfully changed skin").green())?;
            }
            break;
        }
        writeln!(stdout(), "Failed to snipe {}", name)?;
    }
    Ok(())
}
