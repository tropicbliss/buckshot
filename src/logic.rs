use crate::{cli, config, requests, sockets};
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::{join, time};

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

    pub async fn run(&self) {
        self.execute(&self.task).await;
    }

    async fn execute(&self, task: &SnipeTask) {
        let function_id = "Execute";
        let mut count = 0;
        let mut check_filter = true;
        let name_list = if let Some(username_to_snipe) = self.username_to_snipe.to_owned() {
            vec![username_to_snipe]
        } else if !self.config.config.name_queue.is_empty() {
            self.config.config.name_queue.to_owned()
        } else {
            check_filter = false;
            vec![cli::get_username_choice()]
        };
        let requestor = Arc::new(requests::Requests::new());
        if let SnipeTask::Giftcode = task {
            if self.giftcode.is_none() {
                bunt::println!(
                    "{$red}This is a reminder that you should redeem your giftcode before GC sniping.{/$}"
                );
            }
        }
        for username_to_snipe in name_list {
            let username_to_snipe = username_to_snipe.trim();
            count += 1;
            if check_filter && !cli::username_filter_predicate(username_to_snipe) {
                cli::kalm_panik(
                    function_id,
                    &format!("{} is an invalid name.", username_to_snipe),
                );
                continue;
            }
            let requestor = Arc::clone(&requestor);
            if count == 1 {
                println!("Initialising...");
            } else {
                println!("Moving on to next name...");
                println!("Waiting 60 seconds to prevent rate limiting, please stand by."); // As the only publicly available sniper that does name queueing, please tell me if there is an easier way to solve this problem.
                time::sleep(std::time::Duration::from_secs(60)).await;
            }
            let snipe_time = match self.get_snipe_time(&requestor, username_to_snipe).await {
                Some(x) => x,
                None => {
                    continue;
                }
            };
            let access_token = self.setup(&requestor, task).await;
            match task {
                SnipeTask::Giftcode => {
                    if let Some(gc) = &self.giftcode {
                        requestor.redeem_giftcode(gc, &access_token).await;
                    }
                }
                _ => {
                    requestor.check_name_change_eligibility(&access_token).await;
                }
            }
            let offset = if self.config.config.auto_offset {
                match task {
                    SnipeTask::Giftcode => {
                        sockets::auto_offset_calculation_gc(username_to_snipe).await
                    }
                    _ => sockets::auto_offset_calculation_regular(username_to_snipe).await,
                }
            } else {
                self.config.config.offset
            };
            let snipe_status = self
                .snipe(
                    &snipe_time,
                    username_to_snipe,
                    offset,
                    access_token,
                    &requestor,
                    task,
                )
                .await;
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
        cli::exit_program();
    }

    async fn snipe(
        &self,
        droptime: &DateTime<Utc>,
        username_to_snipe: &str,
        offset: i32,
        mut access_token: String,
        requestor: &Arc<requests::Requests>,
        task: &SnipeTask,
    ) -> Option<bool> {
        let droptime = droptime.to_owned();
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime - Utc::now();
        if duration_in_sec < Duration::minutes(1) {
            println!(
                "Sniping {} with an offset of {} ms in ~{} seconds | sniping at {} (utc).",
                username_to_snipe,
                offset,
                duration_in_sec.num_seconds(),
                formatted_droptime
            )
        } else {
            println!(
                "Sniping {} with an offset of {} ms in ~{} minutes | sniping at {} (utc).",
                username_to_snipe,
                offset,
                duration_in_sec.num_minutes(),
                formatted_droptime
            )
        }
        let snipe_time = droptime - Duration::milliseconds(offset as i64);
        let setup_time = snipe_time - Duration::minutes(3);
        access_token = if Utc::now() < setup_time {
            let sleep_duration = match (setup_time - Utc::now()).to_std() {
                Ok(x) => x,
                Err(_) => std::time::Duration::ZERO,
            };
            time::sleep(sleep_duration).await;
            let access_token = self.setup(requestor, task).await;
            access_token
        } else {
            access_token
        };
        match task {
            SnipeTask::Giftcode => self.get_snipe_time(requestor, username_to_snipe).await,
            _ => {
                let (snipe_time, _) = join!(
                    self.get_snipe_time(requestor, username_to_snipe),
                    requestor.check_name_change_eligibility(&access_token)
                );
                snipe_time
            }
        }?;
        bunt::println!("{$green}Signed in to {}.{/$}", self.config.account.email);
        println!("Setup complete.");
        let is_success = match task {
            SnipeTask::Giftcode => {
                sockets::snipe_gc(
                    &snipe_time,
                    username_to_snipe.to_string(),
                    &access_token,
                    self.config.config.spread as i32,
                )
                .await
            }
            _ => {
                sockets::snipe_regular(
                    &snipe_time,
                    username_to_snipe.to_string(),
                    &access_token,
                    self.config.config.spread as i32,
                )
                .await
            }
        };
        if is_success {
            bunt::println!("{$green}Successfully sniped {}!{/$}", username_to_snipe);
            if self.config.config.change_skin {
                requestor.upload_skin(&self.config, &access_token).await;
            }
        } else {
            println!("Failed to snipe {}.", username_to_snipe);
        }
        Some(is_success)
    }

    async fn setup(&self, requestor: &Arc<requests::Requests>, task: &SnipeTask) -> String {
        match task {
            SnipeTask::Mojang => {
                let access_token = requestor
                    .authenticate_mojang(&self.config.account.email, &self.config.account.password)
                    .await;
                if let Some(sq_id) = requestor.get_sq_id(&access_token).await {
                    let answer = [
                        &self.config.account.sq1,
                        &self.config.account.sq2,
                        &self.config.account.sq3,
                    ];
                    requestor.send_sq(&access_token, &sq_id, &answer).await;
                }
                access_token
            }
            _ => {
                requestor
                    .authenticate_microsoft(
                        &self.config.account.email,
                        &self.config.account.password,
                    )
                    .await
            }
        }
    }

    async fn get_snipe_time(
        &self,
        requestor: &Arc<requests::Requests>,
        username_to_snipe: &str,
    ) -> Option<DateTime<Utc>> {
        let function_id = "GetSnipeTime";
        match requestor
            .check_name_availability_time(username_to_snipe)
            .await
        {
            Ok(x) => Some(x),
            Err(requests::NameAvailabilityError::NameNotAvailableError) => {
                cli::kalm_panik(function_id, "Failed to time snipe.");
                None
            }
        }
    }
}
