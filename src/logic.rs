use crate::{cli, config, requests, sockets};
use ansi_term::Colour::{Green, Red};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::io::{stdout, Write};
use std::sync::Arc;
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
}

impl Sniper {
    pub fn new(
        task: SnipeTask,
        username_to_snipe: Option<String>,
        config: config::Config,
        giftcode: Option<String>,
    ) -> Self {
        Self {
            task,
            username_to_snipe,
            config,
            giftcode,
        }
    }

    pub async fn run(&self) -> Result<()> {
        self.execute(&self.task).await?;
        Ok(())
    }

    async fn execute(&self, task: &SnipeTask) -> Result<()> {
        let mut check_filter = true;
        let name_list = if let Some(username_to_snipe) = self.username_to_snipe.clone() {
            vec![username_to_snipe]
        } else if self.config.config.name_queue.is_empty() {
            check_filter = false;
            vec![cli::get_username_choice()?]
        } else {
            self.config.config.name_queue.clone()
        };
        let requestor = Arc::new(requests::Requests::new()?);
        if task == &SnipeTask::Giftcode && self.giftcode.is_none() {
            writeln!(
                stdout(),
                "{}",
                Red.paint("Reminder: You should redeem your giftcode before GC sniping")
            )?;
        }
        for (count, username_to_snipe) in name_list.into_iter().enumerate() {
            let username_to_snipe = username_to_snipe.trim();
            if check_filter && !cli::username_filter_predicate(username_to_snipe) {
                writeln!(
                    stdout(),
                    "{}",
                    Red.paint(format!("{} is an invalid name", username_to_snipe))
                )?;
                continue;
            }
            let requestor = Arc::clone(&requestor);
            if count == 0 {
                writeln!(stdout(), "Initialising...")?;
            } else {
                writeln!(stdout(), "Moving on to next name...")?;
                writeln!(stdout(), "Waiting 20 seconds to prevent rate limiting...")?; // As the only publicly available sniper that does name queueing, please tell me if there is an easier way to solve this problem.
                time::sleep(std::time::Duration::from_secs(20)).await;
            }
            let snipe_time = match requestor
                .check_name_availability_time(username_to_snipe)
                .await?
            {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            let access_token = self.setup(&requestor, task).await?;
            if task == &SnipeTask::Giftcode {
                if let Some(gc) = &self.giftcode {
                    requestor.redeem_giftcode(gc, &access_token).await?;
                }
            } else {
                requestor
                    .check_name_change_eligibility(&access_token)
                    .await?;
            }
            let offset = if self.config.config.auto_offset {
                writeln!(stdout(), "Measuring offset...")?;
                if task == &SnipeTask::Giftcode {
                    sockets::auto_offset_calculator(username_to_snipe, true).await?
                } else {
                    sockets::auto_offset_calculator(username_to_snipe, false).await?
                }
            } else {
                self.config.config.offset
            };
            writeln!(stdout(), "Your offset is: {} ms", offset)?;
            let snipe_status = self
                .snipe(
                    snipe_time,
                    username_to_snipe,
                    offset,
                    access_token,
                    &requestor,
                    task,
                )
                .await?;
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

    async fn snipe(
        &self,
        droptime: DateTime<Utc>,
        username_to_snipe: &str,
        offset: i64,
        mut access_token: String,
        requestor: &Arc<requests::Requests>,
        task: &SnipeTask,
    ) -> Result<Option<bool>> {
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime - Utc::now();
        if duration_in_sec < Duration::minutes(1) {
            writeln!(
                stdout(),
                "Sniping {} in ~{} seconds | sniping at {} (utc)",
                username_to_snipe,
                duration_in_sec.num_seconds(),
                formatted_droptime
            )?;
        } else {
            writeln!(
                stdout(),
                "Sniping {} in ~{} minutes | sniping at {} (utc)",
                username_to_snipe,
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
            access_token = self.setup(requestor, task).await?;
        }
        let stub_time = if task == &SnipeTask::Giftcode {
            requestor
                .check_name_availability_time(username_to_snipe)
                .await?
        } else {
            let (snipe_time, _) = join!(
                requestor.check_name_availability_time(username_to_snipe),
                requestor.check_name_change_eligibility(&access_token)
            );
            snipe_time?
        };
        if stub_time.is_none() {
            return Ok(None);
        }
        writeln!(stdout(), "{}", Green.paint("Successfully signed in"))?;
        writeln!(stdout(), "Setup complete")?;
        let is_success = if task == &SnipeTask::Giftcode {
            sockets::snipe_executor(
                username_to_snipe,
                &access_token,
                self.config.config.spread,
                snipe_time,
                true,
            )
            .await?
        } else {
            sockets::snipe_executor(
                username_to_snipe,
                &access_token,
                self.config.config.spread,
                snipe_time,
                false,
            )
            .await?
        };
        if is_success {
            writeln!(
                stdout(),
                "{}",
                Green.paint(format!("Successfully sniped {}!", username_to_snipe))
            )?;
            if self.config.config.change_skin {
                requestor.upload_skin(&self.config, &access_token).await?;
                writeln!(stdout(), "{}", Green.paint("Successfully changed skin"))?;
            }
        } else {
            writeln!(stdout(), "Failed to snipe {}", username_to_snipe)?;
        }
        Ok(Some(is_success))
    }

    async fn setup(&self, requestor: &Arc<requests::Requests>, task: &SnipeTask) -> Result<String> {
        if task == &SnipeTask::Mojang {
            let access_token = requestor
                .authenticate_mojang(&self.config.account.email, &self.config.account.password)
                .await?;
            if let Some(sq_id) = requestor.get_sq_id(&access_token).await? {
                let answer = [
                    &self.config.account.sq1,
                    &self.config.account.sq2,
                    &self.config.account.sq3,
                ];
                requestor.send_sq(&access_token, &sq_id, &answer).await?;
            }
            Ok(access_token)
        } else {
            requestor
                .authenticate_microsoft(&self.config.account.email, &self.config.account.password)
                .await
        }
    }
}
