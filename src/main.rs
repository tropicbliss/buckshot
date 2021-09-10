mod cli;
mod config;
mod msauth;
mod requests;
mod sockets;

use anyhow::{bail, Context, Result};
use chrono::{Duration, Utc};
use console::style;
use std::thread::sleep;

#[derive(PartialEq)]
pub enum SnipeTask {
    Mojang,
    Microsoft,
    Giftcode,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::new();
    let config = config::Config::new(&args.config_path)
        .with_context(|| format!("Failed to parse {}", args.config_path.display()))?;
    let task = if !config.microsoft_auth {
        if config.gc_snipe {
            println!("{}", style("`microsoft_auth` is set to false yet `gc_snipe` is set to true, defaulting to GC sniping instead").red());
            SnipeTask::Giftcode
        } else {
            SnipeTask::Mojang
        }
    } else if config.gc_snipe {
        SnipeTask::Giftcode
    } else {
        SnipeTask::Microsoft
    };
    if task != SnipeTask::Giftcode && config.account_entry.len() != 1 {
        bail!("You can only provide 1 account in config file as sniper is not set to GC sniping mode");
    }
    let name_list = if let Some(name) = args.name {
        vec![name]
    } else if let Some(x) = config.name_queue.clone() {
        x
    } else {
        let name = cli::get_name_choice().with_context(|| "Failed to get name choice")?;
        vec![name]
    };
    let requestor = requests::Requests::new()?;
    for (count, name) in name_list.into_iter().enumerate() {
        if count != 0 {
            println!("Moving on to next name...");
            println!("Waiting 20 seconds to prevent rate limiting...");
            sleep(std::time::Duration::from_secs(20));
        }
        println!("Initialising...");
        let droptime = if let Some(x) = requestor
            .check_name_availability_time(&name)
            .with_context(|| format!("Failed to get the droptime of {}", name))?
        {
            x
        } else {
            continue;
        };
        let is_gc = task == SnipeTask::Giftcode;
        let executor = sockets::Executor::new(&name, is_gc);
        let offset = if let Some(x) = config.offset {
            x
        } else {
            println!("Calculating offset...");
            executor
                .auto_offset_calculator()
                .await
                .with_context(|| "Failed to calculate offset")?
        };
        println!("Your offset is: {} ms", offset);
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime - Utc::now();
        if duration_in_sec < Duration::minutes(1) {
            println!(
                "Sniping {} in ~{} seconds | sniping at {} (utc)",
                name,
                duration_in_sec.num_seconds(),
                formatted_droptime
            );
        } else {
            println!(
                "Sniping {} in ~{} minutes | sniping at {} (utc)",
                name,
                duration_in_sec.num_minutes(),
                formatted_droptime
            );
        }
        let snipe_time = droptime - Duration::milliseconds(offset);
        let setup_time = snipe_time - Duration::hours(12);
        if Utc::now() < setup_time {
            let sleep_duration = match (setup_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => std::time::Duration::ZERO,
            };
            sleep(sleep_duration);
            if requestor
                .check_name_availability_time(&name)
                .with_context(|| format!("Failed to get the droptime of {}", name))?
                .is_none()
            {
                continue;
            }
        }
        let mut bearer_tokens = Vec::new();
        let mut is_warned = false;
        for account in &config.account_entry {
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
                authenticator.authenticate().with_context(|| {
                    format!(
                        "Failed to authenticate the Microsoft account: {}",
                        account.email
                    )
                })?
            };
            if task == SnipeTask::Giftcode && count == 0 {
                if let Some(gc) = &account.giftcode {
                    requestor
                        .redeem_giftcode(&bearer_token, gc)
                        .with_context(|| {
                            format!(
                                "Failed to redeem giftcode of the account: {}",
                                account.email
                            )
                        })?;
                    println!("{}", style("Successfully redeemed giftcode").green());
                } else if !is_warned {
                    println!(
                        "{}",
                        style("Reminder: You should redeem your giftcode before GC sniping").red()
                    );
                    is_warned = true;
                }
            }
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
            if config.account_entry.len() != 1 {
                println!("Waiting 20 seconds to prevent rate limiting...");
                sleep(std::time::Duration::from_secs(20));
            }
        }
        println!("{}", style("Successfully signed in").green());
        println!("Setup complete");
        match executor
            .snipe_executor(&bearer_tokens, config.spread, snipe_time)
            .await
            .with_context(|| format!("Failed to execute the snipe of {}", name))?
        {
            Some(account_idx) => {
                println!(
                    "{}",
                    style(format!("Successfully sniped {}!", name)).green()
                );
                if let Some(skin) = &config.skin {
                    let skin_model = if skin.slim { "slim" } else { "classic" }.to_string();
                    requestor
                        .upload_skin(&bearer_tokens[account_idx], &skin.skin_path, skin_model)
                        .with_context(|| {
                            format!(
                                "Failed to change the skin of {}",
                                config.account_entry[account_idx].email
                            )
                        })?;
                    println!("{}", style("Successfully changed skin").green());
                }
                break;
            }
            None => {
                println!("Failed to snipe {}", name);
            }
        }
    }
    Ok(())
}
