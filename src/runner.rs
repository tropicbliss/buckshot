// The ultimate offender of DRY, ladies and gentlemen

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
}

impl Sniper {
    pub fn new(task: SnipeTask, username_to_snipe: Option<String>, config: config::Config) -> Self {
        Self {
            task,
            username_to_snipe,
            config,
        }
    }

    pub async fn run(&self) {
        self.execute(&self.task).await;
    }

    async fn execute(&self, task: &SnipeTask) {
        let mut count = 0;
        let mut is_success = false;
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
        for username_to_snipe in name_list {
            count += 1;
            if check_filter && !cli::username_filter_predicate(&username_to_snipe) {
                cli::kalm_panik(
                    "Main",
                    &format!("{} is an invalid username.", username_to_snipe),
                );
                continue;
            }
            let requestor = Arc::clone(&requestor);
            if count == 1 {
                println!("Initialising...");
            } else {
                println!("Moving on to next name...");
            }
            let snipe_time = match self.get_snipe_time(&requestor, &username_to_snipe).await {
                Some(x) => x.droptime,
                None => continue,
            };
            let access_token = self.setup(&requestor, task).await;
            match task {
                SnipeTask::Giftcode => {
                    let giftcode = cli::get_giftcode();
                    if let Some(gc) = giftcode {
                        requestor.redeem_giftcode(&gc, &access_token).await;
                    }
                }
                _ => {
                    requestor.check_name_change_eligibility(&access_token).await;
                }
            }
            let offset = if self.config.config.auto_offset {
                match task {
                    SnipeTask::Giftcode => {
                        sockets::auto_offset_calculation_gc(&username_to_snipe).await
                    }
                    _ => sockets::auto_offset_calculation_regular(&username_to_snipe).await,
                }
            } else {
                self.config.config.offset
            };
            println!("Your offset is: {} ms.", offset);
            let snipe_status = self
                .snipe(
                    &snipe_time,
                    &username_to_snipe,
                    offset,
                    &access_token,
                    &requestor,
                    task,
                )
                .await;
            let snipe_status = match snipe_status {
                Some(x) => x,
                None => {
                    cli::kalm_panik(
                        "GetDrop",
                        &format!(
                            "The name {} is not available or has already dropped.",
                            username_to_snipe
                        ),
                    );
                    continue;
                }
            };
            if snipe_status {
                is_success = true;
                break;
            }
        }
        if !is_success && count > 1 {
            println!("Unfortunately, you did not snipe a name.");
        }
        cli::exit_program();
    }

    async fn snipe(
        &self,
        droptime: &DateTime<Utc>,
        username_to_snipe: &str,
        offset: i32,
        access_token: &str,
        requestor: &Arc<requests::Requests>,
        task: &SnipeTask,
    ) -> Option<bool> {
        let droptime = droptime.to_owned();
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime - Utc::now();
        if duration_in_sec < Duration::minutes(1) {
            println!(
                "Sniping {} in ~{} seconds | sniping at {} (utc).",
                username_to_snipe,
                duration_in_sec.num_seconds(),
                formatted_droptime
            )
        } else {
            println!(
                "Sniping {} in ~{} minutes | sniping at {} (utc).",
                username_to_snipe,
                duration_in_sec.num_minutes(),
                formatted_droptime
            )
        }
        let snipe_time = droptime - Duration::milliseconds(offset as i64);
        let setup_time = snipe_time - Duration::minutes(12);
        let access_token = if Utc::now() < setup_time {
            time::sleep((setup_time - Utc::now()).to_std().unwrap()).await;
            let access_token = self.setup(&requestor, task).await;
            access_token
        } else {
            access_token.to_string()
        };
        let namemc_data = match task {
            SnipeTask::Giftcode => {
                requestor
                    .check_name_availability_time(&username_to_snipe)
                    .await
            }
            _ => {
                let (snipe_time, _) = join!(
                    requestor.check_name_availability_time(&username_to_snipe),
                    requestor.check_name_change_eligibility(&access_token)
                );
                snipe_time
            }
        };
        let searches = match namemc_data {
            Ok(x) => x.searches,
            Err(_) => return None,
        };
        bunt::println!(
            "{$green}Signed in to {} successfully.{/$}",
            self.config.account.username
        );
        let is_success = match task {
            SnipeTask::Giftcode => {
                sockets::snipe_gc(
                    &snipe_time,
                    username_to_snipe,
                    &access_token,
                    self.config.config.spread as i32,
                )
                .await
            }
            _ => {
                sockets::snipe_regular(
                    &snipe_time,
                    username_to_snipe,
                    &access_token,
                    self.config.config.spread as i32,
                )
                .await
            }
        };
        if is_success {
            bunt::println!(
                "{$green}Successfully sniped {} with {} searches!{/$}",
                username_to_snipe,
                searches
            );
            if self.config.config.change_skin {
                requestor.upload_skin(&self.config, &access_token).await;
            }
        }
        Some(is_success)
    }

    async fn setup(&self, requestor: &Arc<requests::Requests>, task: &SnipeTask) -> String {
        match task {
            SnipeTask::Mojang => {
                let access_token = requestor
                    .authenticate_mojang(
                        &self.config.account.username,
                        &self.config.account.password,
                    )
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
                let mut count = 0;
                loop {
                    count += 1;
                    match requestor
                        .authenticate_microsoft(
                            &self.config.account.username,
                            &self.config.account.password,
                        )
                        .await
                    {
                        Ok(x) => break x,
                        Err(requests::AuthenicationError::RetryableAuthenticationError) => {
                            cli::kalm_panik(
                                "MicroAuth",
                                &format!(
                                    "Authentication error. Retrying in 10 seconds. Attempt(s): {}.",
                                    count
                                ),
                            );
                            time::sleep(std::time::Duration::from_secs(10)).await;
                            if count == 3 {
                                cli::pretty_panik(
                                    "MicroAuth",
                                    "Authentication failed due to an unknown server error.",
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    async fn get_snipe_time(
        &self,
        requestor: &Arc<requests::Requests>,
        username_to_snipe: &str,
    ) -> Option<requests::NameMC> {
        match requestor
            .check_name_availability_time(&username_to_snipe)
            .await
        {
            Ok(x) => Some(x),
            Err(requests::NameAvailabilityError::NameNotAvailableError) => {
                cli::kalm_panik(
                        "GetDrop",
                        &format!(
                            "Failed to time snipe. The name {} is not available, has already dropped, or is not cached in the API server for various circumstances.",
                            username_to_snipe
                        ),
                    );
                None
            }
        }
    }
}
