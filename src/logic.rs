use crate::{cli, config, requests, sockets};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use console::{style, Emoji};
use indicatif::ProgressBar;
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

pub struct Sniper {
    task: SnipeTask,
    username_to_snipe: Option<String>,
    config: config::Config,
    giftcode: Option<String>,
    requestor: requests::Requests,
    name: String,
    offset: i64,
    executor: sockets::Executor,
}

impl Sniper {
    pub fn new(
        task: SnipeTask,
        username_to_snipe: Option<String>,
        config: config::Config,
        giftcode: Option<String>,
    ) -> Result<Self> {
        let email = config.account.email.clone();
        let password = config.account.password.clone();
        Ok(Self {
            task,
            username_to_snipe,
            config,
            giftcode,
            requestor: requests::Requests::new(email, password)?,
            name: String::new(),
            offset: 0,
            executor: sockets::Executor::new(String::new(), false),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        static HOURGLASS: Emoji<'_, '_> = Emoji("\u{231b} ", "");
        static SPARKLE: Emoji<'_, '_> = Emoji("\u{2728} ", ":-) ");
        let mut check_filter = true;
        let name_list = if let Some(username_to_snipe) = self.username_to_snipe.clone() {
            vec![username_to_snipe]
        } else if self.config.config.name_queue.is_empty() {
            check_filter = false;
            vec![cli::get_username_choice()
                .with_context(|| anyhow!("Failed to get username choice"))?]
        } else {
            self.config.config.name_queue.clone()
        };
        for (count, username) in name_list.into_iter().enumerate() {
            self.name = username.trim().to_string();
            let is_gc = self.task == SnipeTask::Giftcode;
            self.executor = sockets::Executor::new(self.name.clone(), is_gc);
            if check_filter && !cli::username_filter_predicate(&self.name) {
                writeln!(
                    stdout(),
                    "{}",
                    style(format!("{} is an invalid name", self.name)).red()
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
            let snipe_time = if let Some(x) = self
                .requestor
                .check_name_availability_time(&self.name)
                .with_context(|| anyhow!("Failed to get droptime"))?
            {
                progress_bar.inc(25);
                x
            } else {
                progress_bar.abandon();
                continue;
            };
            self.setup()
                .with_context(|| anyhow!("Failed to run authenticator"))?;
            progress_bar.inc(25);
            if self.task == SnipeTask::Giftcode && count == 0 {
                if let Some(gc) = &self.giftcode {
                    self.requestor.redeem_giftcode(gc)?;
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
                self.requestor
                    .check_name_change_eligibility()
                    .with_context(|| anyhow!("Failed to check name change eligibility"))?;
            }
            progress_bar.inc(25);
            self.get_offset()
                .await
                .with_context(|| anyhow!("Failed to get offset"))?;
            progress_bar.inc(25);
            progress_bar.finish();
            writeln!(
                stdout(),
                "{}Initialisation complete. Your offset is: {} ms",
                SPARKLE,
                self.offset
            )?;
            let snipe_status = self.snipe(snipe_time).await?;
            let snipe_status = match snipe_status {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            if snipe_status {
                break;
            }
        }
        Ok(())
    }

    async fn snipe(&mut self, droptime: DateTime<Utc>) -> Result<Option<bool>> {
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime - Utc::now();
        if duration_in_sec < Duration::minutes(1) {
            writeln!(
                stdout(),
                "Sniping {} in ~{} seconds | sniping at {} (utc)",
                self.name,
                duration_in_sec.num_seconds(),
                formatted_droptime
            )?;
        } else {
            writeln!(
                stdout(),
                "Sniping {} in ~{} minutes | sniping at {} (utc)",
                self.name,
                duration_in_sec.num_minutes(),
                formatted_droptime
            )?;
        }
        let snipe_time = droptime - Duration::milliseconds(self.offset);
        let setup_time = snipe_time - Duration::minutes(720);
        if Utc::now() < setup_time {
            let sleep_duration = match (setup_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => std::time::Duration::ZERO,
            };
            sleep(sleep_duration);
            self.setup()
                .with_context(|| anyhow!("Failed to run authenticator"))?;
        }
        writeln!(stdout(), "{}", style("Successfully signed in").green())?;
        if self
            .requestor
            .check_name_availability_time(&self.name)
            .with_context(|| anyhow!("Failed to get droptime"))?
            .is_none()
        {
            return Ok(None);
        }
        if self.task != SnipeTask::Giftcode {
            self.requestor
                .check_name_change_eligibility()
                .with_context(|| anyhow!("Failed to check name change eligibility"))?;
        };
        writeln!(stdout(), "Setup complete")?;
        let is_success = self
            .executor
            .snipe_executor(
                &self.requestor.bearer_token,
                self.config.config.spread,
                snipe_time,
            )
            .await
            .with_context(|| anyhow!("Failed to execute snipe"))?;
        if is_success {
            writeln!(
                stdout(),
                "{}",
                style(format!("Successfully sniped {}!", self.name)).green()
            )?;
            if self.config.config.change_skin {
                self.requestor
                    .upload_skin(
                        &self.config.config.skin_filename,
                        self.config.config.skin_model.clone(),
                    )
                    .with_context(|| anyhow!("Failed to upload skin"))?;
                writeln!(stdout(), "{}", style("Successfully changed skin").green())?;
            }
        } else {
            writeln!(stdout(), "Failed to snipe {}", self.name)?;
        }
        Ok(Some(is_success))
    }

    fn setup(&mut self) -> Result<()> {
        if self.task == SnipeTask::Mojang {
            self.requestor
                .authenticate_mojang()
                .with_context(|| anyhow!("Failed to authenticate Mojang account"))?;
            if self
                .requestor
                .get_sq_id()
                .with_context(|| anyhow!("Failed to get SQ IDs."))?
            {
                let answer = [
                    &self.config.account.sq1,
                    &self.config.account.sq2,
                    &self.config.account.sq3,
                ];
                self.requestor
                    .send_sq(answer)
                    .with_context(|| anyhow!("Failed to send SQ answers"))?;
            }
        } else {
            self.requestor
                .authenticate_microsoft()
                .with_context(|| anyhow!("Failed to authenticate Microsoft account"))?;
        }
        Ok(())
    }

    async fn get_offset(&mut self) -> Result<()> {
        self.offset = if self.config.config.auto_offset {
            self.executor
                .auto_offset_calculator()
                .await
                .with_context(|| anyhow!("Failed to calculate offset"))?
        } else {
            self.config.config.offset
        };
        Ok(())
    }
}
