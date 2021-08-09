use crate::{cli, config, requests, sockets};
use ansi_term::Colour::{Green, Red};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::io::{stdout, Write};
use tokio::{join, time};

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
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.execute().await?;
        Ok(())
    }

    async fn execute(&mut self) -> Result<()> {
        let mut check_filter = true;
        let name_list = if let Some(username_to_snipe) = self.username_to_snipe.clone() {
            vec![username_to_snipe]
        } else if self.config.config.name_queue.is_empty() {
            check_filter = false;
            vec![cli::get_username_choice()?]
        } else {
            self.config.config.name_queue.clone()
        };
        if self.task == SnipeTask::Giftcode && self.giftcode.is_none() {
            writeln!(
                stdout(),
                "{}",
                Red.paint("Reminder: You should redeem your giftcode before GC sniping")
            )?;
        }
        for (count, username) in name_list.into_iter().enumerate() {
            self.name = username.trim().to_string();
            if check_filter && !cli::username_filter_predicate(&self.name) {
                writeln!(
                    stdout(),
                    "{}",
                    Red.paint(format!("{} is an invalid name", self.name))
                )?;
                continue;
            }
            if count == 0 {
                writeln!(stdout(), "Initialising...")?;
            } else {
                writeln!(stdout(), "Moving on to next name...")?;
                writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?; // As the only publicly available sniper that does name queueing, please tell me if there is an easier way to solve this problem.
                time::sleep(std::time::Duration::from_secs(20)).await;
            }
            let snipe_time = if let Some(x) = self
                .requestor
                .check_name_availability_time(&self.name)
                .await?
            {
                x
            } else {
                continue;
            };
            self.setup().await?;
            if self.task == SnipeTask::Giftcode && count == 0 {
                if let Some(gc) = &self.giftcode {
                    self.requestor.redeem_giftcode(gc).await?;
                    writeln!(
                        stdout(),
                        "{}",
                        Green.paint("Successfully redeemed giftcode")
                    )?;
                }
            } else {
                self.requestor.check_name_change_eligibility().await?;
            }
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
        let is_gc = self.task == SnipeTask::Giftcode;
        let executor = sockets::Executor::new(self.name.clone(), is_gc);
        let offset = if self.config.config.auto_offset {
            writeln!(stdout(), "Measuring offset...")?;
            executor.auto_offset_calculator().await?
        } else {
            self.config.config.offset
        };
        writeln!(stdout(), "Your offset is: {} ms", offset)?;
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
        let snipe_time = droptime - Duration::milliseconds(offset);
        let setup_time = snipe_time - Duration::minutes(3);
        if Utc::now() < setup_time {
            let sleep_duration = match (setup_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => std::time::Duration::ZERO,
            };
            time::sleep(sleep_duration).await;
            self.setup().await?;
        }
        let stub_time = if self.task == SnipeTask::Giftcode {
            self.requestor
                .check_name_availability_time(&self.name)
                .await?
        } else {
            let (snipe_time, _) = join!(
                self.requestor.check_name_availability_time(&self.name),
                self.requestor.check_name_change_eligibility()
            );
            snipe_time?
        };
        if stub_time.is_none() {
            return Ok(None);
        }
        writeln!(stdout(), "{}", Green.paint("Successfully signed in"))?;
        writeln!(stdout(), "Setup complete")?;
        let is_success = executor
            .snipe_executor(
                &self.requestor.bearer_token,
                self.config.config.spread,
                snipe_time,
            )
            .await?;
        if is_success {
            writeln!(
                stdout(),
                "{}",
                Green.paint(format!("Successfully sniped {}!", self.name))
            )?;
            if self.config.config.change_skin {
                self.requestor
                    .upload_skin(
                        &self.config.config.skin_filename,
                        self.config.config.skin_model.clone(),
                    )
                    .await?;
                writeln!(stdout(), "{}", Green.paint("Successfully changed skin"))?;
            }
        } else {
            writeln!(stdout(), "Failed to snipe {}", self.name)?;
        }
        Ok(Some(is_success))
    }

    async fn setup(&mut self) -> Result<()> {
        if self.task == SnipeTask::Mojang {
            self.requestor.authenticate_mojang().await?;
            if self.requestor.get_sq_id().await? {
                let answer = [
                    &self.config.account.sq1,
                    &self.config.account.sq2,
                    &self.config.account.sq3,
                ];
                self.requestor.send_sq(answer).await?;
            }
        } else {
            self.requestor.authenticate_microsoft().await?;
        }
        Ok(())
    }
}
