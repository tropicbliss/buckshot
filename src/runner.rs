use crate::{cli, config, requests};
use chrono::{DateTime, Duration, Utc};
use tokio::{join, task, time};

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
        match self.task {
            SnipeTask::Mojang => self.run_mojang().await,
            SnipeTask::Microsoft => self.run_msa().await,
            SnipeTask::Giftcode => self.run_gc().await,
        }
    }

    async fn run_mojang(&self) {
        let requestor = requests::Requests::new();
        let (access_token, auth_time) = self.setup_mojang(&requestor).await;
        let (snipe_time, username_to_snipe) =
            if let Some(username_to_snipe) = self.username_to_snipe.clone() {
                let (snipe_time, _) = join!(
                    requestor.check_name_availability_time(&username_to_snipe, auth_time),
                    requestor.check_name_change_eligibility(&access_token)
                );
                (snipe_time, username_to_snipe.clone())
            } else {
                let username_to_snipe = cli::get_username_choice();
                let (snipe_time, _) = join!(
                    requestor.check_name_availability_time(&username_to_snipe, auth_time),
                    requestor.check_name_change_eligibility(&access_token)
                );
                (snipe_time, username_to_snipe)
            };
        let offset = if self.config.config.auto_offset {
            requestor
                .auto_offset_calculation_regular(&username_to_snipe)
                .await
        } else {
            cli::get_offset()
        };
        self.snipe_mojang(
            snipe_time,
            username_to_snipe,
            offset,
            &access_token,
            requestor,
        )
        .await;
    }

    async fn run_msa(&self) {
        let requestor = requests::Requests::new();
        let (access_token, auth_time) = self.setup_msa(&requestor).await;
        let (snipe_time, username_to_snipe) =
            if let Some(username_to_snipe) = self.username_to_snipe.clone() {
                let (snipe_time, _) = join!(
                    requestor.check_name_availability_time(&username_to_snipe, auth_time),
                    requestor.check_name_change_eligibility(&access_token)
                );
                (snipe_time, username_to_snipe.clone())
            } else {
                let username_to_snipe = cli::get_username_choice();
                let (snipe_time, _) = join!(
                    requestor.check_name_availability_time(&username_to_snipe, auth_time),
                    requestor.check_name_change_eligibility(&access_token)
                );
                (snipe_time, username_to_snipe)
            };
        let offset = if self.config.config.auto_offset {
            requestor
                .auto_offset_calculation_regular(&username_to_snipe)
                .await
        } else {
            cli::get_offset()
        };
        self.snipe_msa(
            snipe_time,
            username_to_snipe,
            offset,
            &access_token,
            requestor,
        )
        .await;
    }

    async fn run_gc(&self) {
        let requestor = requests::Requests::new();
        let (access_token, auth_time) = self.setup_msa(&requestor).await;
        let giftcode = cli::get_giftcode();
        let (snipe_time, username_to_snipe) =
            if let Some(username_to_snipe) = self.username_to_snipe.clone() {
                if let Some(gc) = giftcode {
                    let (snipe_time, _) = join!(
                        requestor.check_name_availability_time(&username_to_snipe, auth_time),
                        requestor.redeem_giftcode(&gc, &access_token)
                    );
                    (snipe_time, username_to_snipe)
                } else {
                    (
                        requestor
                            .check_name_availability_time(&username_to_snipe, auth_time)
                            .await,
                        username_to_snipe,
                    )
                }
            } else {
                if let Some(gc) = giftcode {
                    let username_to_snipe = cli::get_username_choice();
                    let (snipe_time, _) = join!(
                        requestor.check_name_availability_time(&username_to_snipe, auth_time),
                        requestor.redeem_giftcode(&gc, &access_token)
                    );
                    (snipe_time, username_to_snipe)
                } else {
                    let username_to_snipe = cli::get_username_choice();
                    (
                        requestor
                            .check_name_availability_time(&username_to_snipe, auth_time)
                            .await,
                        username_to_snipe,
                    )
                }
            };
        let offset = if self.config.config.auto_offset {
            requestor
                .auto_offset_calculation_gc(&username_to_snipe)
                .await
        } else {
            cli::get_offset()
        };
        self.snipe_gc(
            snipe_time,
            username_to_snipe,
            offset,
            &access_token,
            requestor,
        )
        .await;
    }

    async fn snipe_mojang(
        &self,
        droptime: DateTime<Utc>,
        username_to_snipe: String,
        offset: i32,
        access_token: &str,
        requestor: requests::Requests,
    ) {
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime.signed_duration_since(Utc::now());
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
            time::sleep(
                setup_time
                    .signed_duration_since(Utc::now())
                    .to_std()
                    .unwrap(),
            )
            .await;
            let (access_token, auth_time) = self.setup_mojang(&requestor).await;
            join!(
                requestor.check_name_availability_time(&username_to_snipe, auth_time),
                requestor.check_name_change_eligibility(&access_token)
            );
            bunt::println!("{$green}Signed in to {}.{/$}", self.config.account.username);
            access_token
        } else {
            access_token.to_string()
        };
        let is_success = self
            .snipe_regular(
                snipe_time,
                username_to_snipe.clone(),
                access_token.to_string(),
            )
            .await;
        if is_success {
            bunt::println!("{$green}Successfully sniped {}!{/$}", username_to_snipe);
            if self.config.config.change_skin {
                requestor.upload_skin(&self.config, &access_token).await;
            }
        }
        cli::exit_program();
    }

    async fn snipe_msa(
        &self,
        droptime: DateTime<Utc>,
        username_to_snipe: String,
        offset: i32,
        access_token: &str,
        requestor: requests::Requests,
    ) {
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime.signed_duration_since(Utc::now());
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
        if Utc::now() < setup_time {
            time::sleep(
                setup_time
                    .signed_duration_since(Utc::now())
                    .to_std()
                    .unwrap(),
            )
            .await;
            join!(
                requestor.check_name_availability_time(&username_to_snipe, None),
                requestor.check_name_change_eligibility(&access_token)
            );
            bunt::println!("{$green}Signed in to {}.{/$}", self.config.account.username);
        }
        let is_success = self
            .snipe_regular(
                snipe_time,
                username_to_snipe.clone(),
                access_token.to_string(),
            )
            .await;
        if is_success {
            bunt::println!("{$green}Successfully sniped {}!{/$}", username_to_snipe);
            if self.config.config.change_skin {
                requestor.upload_skin(&self.config, &access_token).await;
            }
        }
        cli::exit_program();
    }

    async fn snipe_gc(
        &self,
        droptime: DateTime<Utc>,
        username_to_snipe: String,
        offset: i32,
        access_token: &str,
        requestor: requests::Requests,
    ) {
        let formatted_droptime = droptime.format("%F %T");
        let duration_in_sec = droptime.signed_duration_since(Utc::now());
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
        if Utc::now() < setup_time {
            time::sleep(
                setup_time
                    .signed_duration_since(Utc::now())
                    .to_std()
                    .unwrap(),
            )
            .await;
            join!(
                requestor.check_name_availability_time(&username_to_snipe, None),
                requestor.check_name_change_eligibility(&access_token)
            );
            bunt::println!("{$green}Signed in to {}.{/$}", self.config.account.username);
        }
        let is_success = self
            .snipe_giftcode(
                snipe_time,
                username_to_snipe.clone(),
                access_token.to_string(),
            )
            .await;
        if is_success {
            bunt::println!("{$green}Successfully sniped {}!{/$}", username_to_snipe);
            if self.config.config.change_skin {
                requestor.upload_skin(&self.config, &access_token).await;
            }
        }
        cli::exit_program();
    }

    async fn setup_mojang(
        &self,
        requestor: &requests::Requests,
    ) -> (String, Option<DateTime<Utc>>) {
        let (access_token, auth_time) = requestor
            .authenticate_mojang(&self.config.account.username, &self.config.account.password)
            .await;
        if requestor.check_sq(&access_token).await {
            if let Some(sq_id) = requestor.get_sq_id(&access_token).await {
                let answer = [
                    &self.config.account.sq1,
                    &self.config.account.sq2,
                    &self.config.account.sq3,
                ];
                requestor.send_sq(&access_token, sq_id, answer).await;
            }
        }
        (access_token, auth_time)
    }

    async fn setup_msa(&self, requestor: &requests::Requests) -> (String, Option<DateTime<Utc>>) {
        requestor.authenticate_microsoft()
    }

    async fn snipe_regular(
        &self,
        snipe_time: DateTime<Utc>,
        username_to_snipe: String,
        access_token: String,
    ) -> bool {
        let mut handle_vec: Vec<task::JoinHandle<u16>> = Vec::new();
        let mut status_vec: Vec<u16> = Vec::new();
        let mut initial_spread = 0;
        for _ in 0..2 {
            let username_to_snipe = username_to_snipe.clone();
            let access_token = access_token.clone();
            let handle = task::spawn(async move {
                requests::snipe_task_regular(
                    snipe_time,
                    username_to_snipe,
                    access_token,
                    initial_spread,
                )
                .await
            });
            handle_vec.push(handle);
            initial_spread += self.config.config.spread as i32;
        }
        for handle in handle_vec {
            status_vec.push(handle.await.unwrap());
        }
        status_vec.contains(&200)
    }

    async fn snipe_giftcode(
        &self,
        snipe_time: DateTime<Utc>,
        username_to_snipe: String,
        access_token: String,
    ) -> bool {
        let mut handle_vec: Vec<task::JoinHandle<u16>> = Vec::new();
        let mut status_vec: Vec<u16> = Vec::new();
        let mut initial_spread = 0;
        for _ in 0..6 {
            let username_to_snipe = username_to_snipe.clone();
            let access_token = access_token.clone();
            let handle = task::spawn(async move {
                requests::snipe_task_gc(snipe_time, username_to_snipe, access_token, initial_spread)
                    .await
            });
            handle_vec.push(handle);
            initial_spread += self.config.config.spread as i32;
        }
        for handle in handle_vec {
            status_vec.push(handle.await.unwrap());
        }
        status_vec.contains(&200)
    }
}
