#![allow(clippy::too_many_lines)]

use crate::{cli, config, requests, sockets};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
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

pub async fn run(
    username_to_snipe: Option<String>,
    config: config::Config,
    giftcode: Option<String>,
) -> Result<()> {
    let task = if !config.config.microsoft_auth {
        if config.config.gc_snipe {
            writeln!(stdout(), "{}", style("`microsoft_auth` is set to false yet `gc_snipe` is set to true, defaulting to GC sniping instead").red())?;
            SnipeTask::Giftcode
        } else {
            SnipeTask::Mojang
        }
    } else if config.config.gc_snipe {
        SnipeTask::Giftcode
    } else {
        SnipeTask::Microsoft
    };
    static HOURGLASS: Emoji<'_, '_> = Emoji("\u{231b} ", "");
    static SPARKLE: Emoji<'_, '_> = Emoji("\u{2728} ", ":-) ");
    let email = config.account.email.clone();
    let password = config.account.password.clone();
    let mut requestor = requests::Requests::new(email, password)?;
    let mut check_filter = true;
    let name_list = if let Some(username_to_snipe) = username_to_snipe {
        vec![username_to_snipe]
    } else if config.config.name_queue.is_empty() {
        check_filter = false;
        vec![cli::get_username_choice().with_context(|| "Failed to get username choice")?]
    } else {
        config.config.name_queue
    };
    for (count, username) in name_list.into_iter().enumerate() {
        let name = username.trim().to_string();
        if check_filter && !cli::username_filter_predicate(&name) {
            writeln!(
                stdout(),
                "{}",
                style(format!("{} is an invalid name", name)).red()
            )?;
            continue;
        }
        if count != 0 {
            writeln!(stdout(), "Moving on to next name...")?;
            writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?; // As the only publicly available sniper that does name queueing, please tell me if there is an easier way to solve this problem.
            sleep(std::time::Duration::from_secs(20));
        }
        writeln!(stdout(), "{}Initialising...", HOURGLASS)?;
        let progress_bar = ProgressBar::new(100);
        let progress_bar_style = ProgressStyle::default_bar()
            .progress_chars("= ")
            .template("{wide_bar} {percent}%");
        progress_bar.set_style(progress_bar_style);
        let droptime = if let Some(x) = requestor
            .check_name_availability_time(&name)
            .with_context(|| "Failed to get droptime")?
        {
            progress_bar.inc(25);
            x
        } else {
            progress_bar.abandon();
            continue;
        };
        let mut bearer_token = if task == SnipeTask::Mojang {
            let bearer_token = requestor
                .authenticate_mojang()
                .with_context(|| "Failed to authenticate Mojang account")?;
            if let Some(questions) = requestor
                .get_questions(&bearer_token)
                .with_context(|| "Failed to get SQ IDs.")?
            {
                let answers = [
                    &config.account.sq1,
                    &config.account.sq2,
                    &config.account.sq3,
                ];
                requestor
                    .send_answers(&bearer_token, questions, answers)
                    .with_context(|| "Failed to send SQ answers")?;
            }
            bearer_token
        } else {
            requestor
                .authenticate_microsoft()
                .with_context(|| "Failed to authenticate Microsoft account")?
        };
        progress_bar.inc(25);
        if task == SnipeTask::Giftcode && count == 0 {
            if let Some(gc) = &giftcode {
                requestor.redeem_giftcode(&bearer_token, gc)?;
                writeln!(
                    stdout(),
                    "{}",
                    style("Successfully redeemed giftcode").green()
                )?;
            } else {
                writeln!(
                    stdout(),
                    "{}",
                    style("Reminder: You should redeem your giftcode before GC sniping").red()
                )?;
            }
        } else {
            requestor
                .check_name_change_eligibility(&bearer_token)
                .with_context(|| "Failed to check name change eligibility")?;
        }
        progress_bar.inc(25);
        let is_gc = task == SnipeTask::Giftcode;
        let executor = sockets::Executor::new(name.clone(), is_gc);
        let offset = if config.config.auto_offset {
            executor
                .auto_offset_calculator()
                .await
                .with_context(|| "Failed to calculate offset")?
        } else {
            config.config.offset
        };
        progress_bar.inc(25);
        progress_bar.finish();
        writeln!(
            stdout(),
            "{}Initialisation complete. Your offset is: {} ms",
            SPARKLE,
            offset
        )?;
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime - Utc::now();
        if duration_in_sec < Duration::minutes(1) {
            writeln!(
                stdout(),
                "Sniping {} in ~{} seconds | sniping at {} (utc)",
                name,
                duration_in_sec.num_seconds(),
                formatted_droptime
            )?;
        } else {
            writeln!(
                stdout(),
                "Sniping {} in ~{} minutes | sniping at {} (utc)",
                name,
                duration_in_sec.num_minutes(),
                formatted_droptime
            )?;
        }
        let snipe_time = droptime - Duration::milliseconds(offset);
        let setup_time = snipe_time - Duration::minutes(720);
        if Utc::now() < setup_time {
            let sleep_duration = match (setup_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => std::time::Duration::ZERO,
            };
            sleep(sleep_duration);
            bearer_token = if task == SnipeTask::Mojang {
                let bearer_token = requestor
                    .authenticate_mojang()
                    .with_context(|| "Failed to authenticate Mojang account")?;
                if let Some(questions) = requestor
                    .get_questions(&bearer_token)
                    .with_context(|| "Failed to get SQ IDs.")?
                {
                    let answers = [
                        &config.account.sq1,
                        &config.account.sq2,
                        &config.account.sq3,
                    ];
                    requestor
                        .send_answers(&bearer_token, questions, answers)
                        .with_context(|| "Failed to send SQ answers")?;
                }
                bearer_token
            } else {
                requestor
                    .authenticate_microsoft()
                    .with_context(|| "Failed to authenticate Microsoft account")?
            };
        }
        writeln!(stdout(), "{}", style("Successfully signed in").green())?;
        if requestor
            .check_name_availability_time(&name)
            .with_context(|| "Failed to get droptime")?
            .is_none()
        {
            continue;
        }
        if task != SnipeTask::Giftcode {
            requestor
                .check_name_change_eligibility(&bearer_token)
                .with_context(|| "Failed to check name change eligibility")?;
        };
        writeln!(stdout(), "Setup complete")?;
        let is_success = executor
            .snipe_executor(&bearer_token, config.config.spread, snipe_time)
            .await
            .with_context(|| "Failed to execute snipe")?;
        if is_success {
            writeln!(
                stdout(),
                "{}",
                style(format!("Successfully sniped {}!", name)).green()
            )?;
            if config.config.change_skin {
                requestor
                    .upload_skin(
                        &bearer_token,
                        &config.config.skin_filename,
                        config.config.skin_model.clone(),
                    )
                    .with_context(|| "Failed to upload skin")?;
                writeln!(stdout(), "{}", style("Successfully changed skin").green())?;
            }
        } else {
            writeln!(stdout(), "Failed to snipe {}", name)?;
        }
        if is_success {
            break;
        }
    }
    Ok(())
}
