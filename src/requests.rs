use ansi_term::Colour::Red;
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{header::ACCEPT, Body, Client};
use serde_json::{json, Value};
use std::{
    convert::TryInto,
    io::{stdout, Write},
    time::Duration,
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

pub struct Requests {
    client: Client,
    pub bearer_token: String,
    email: String,
    password: String,
    sqid: [i64; 3],
}

impl Requests {
    pub fn new(email: String, password: String) -> Result<Self> {
        if email.is_empty() || password.is_empty() {
            bail!("No email or password provided");
        }
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .user_agent("Sniper")
                .tcp_keepalive(Some(Duration::from_secs(5)))
                .use_rustls_tls()
                .build()?,
            bearer_token: String::new(),
            email,
            password,
            sqid: [0; 3],
        })
    }

    pub async fn authenticate_mojang(&mut self) -> Result<()> {
        let post_json = json!({
            "username": self.email,
            "password": self.password
        });
        let res = self
            .client
            .post("https://authserver.mojang.com/authenticate")
            .json(&post_json)
            .send()
            .await?;
        let status = res.status();
        match status.as_u16() {
            200 => {
                let v: Value = serde_json::from_str(&res.text().await?)?;
                let bearer_token = v["accessToken"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `accessToken` from JSON"))?
                    .to_string();
                self.bearer_token = bearer_token;
            }
            403 => {
                bail!("Incorrect email or password");
            }
            _ => {
                bail!(status);
            }
        }
        Ok(())
    }

    pub async fn authenticate_microsoft(&mut self) -> Result<()> {
        let post_json = json!({
            "email": self.email,
            "password": self.password
        });
        let res = self
            .client
            .post("https://auth.buckshotrs.com")
            .json(&post_json)
            .send()
            .await?;
        let status = res.status();
        match status.as_u16() {
            200 => {
                let body = res.text().await?;
                let v: Value = serde_json::from_str(&body)?;
                let bearer_token = v["bearer_token"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `bearer_token` from JSON"))?
                    .to_string();
                self.bearer_token = bearer_token;
            }
            400 => {
                let body = res.text().await?;
                let v: Value = serde_json::from_str(&body)?;
                let err = v["detail"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `detail` from JSON"))?;
                bail!("{}", err);
            }
            _ => {
                bail!(status);
            }
        }
        Ok(())
    }

    pub async fn get_sq_id(&mut self) -> Result<bool> {
        let res = self
            .client
            .get("https://api.mojang.com/user/security/challenges")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!(status);
        }
        let body = res.text().await?;
        if body == "[]" {
            Ok(false)
        } else {
            let v: Value = serde_json::from_str(&body)?;
            let sq_array = v
                .as_array()
                .ok_or_else(|| anyhow!("Unable to parse JSON array"))?;
            let mut sqid_array = Vec::new();
            for item in sq_array {
                let id = item["answer"]["id"]
                    .as_i64()
                    .ok_or_else(|| anyhow!("Unable to parse `answer` or `id` from JSON"))?;
                sqid_array.push(id);
            }
            self.sqid = sqid_array
                .try_into()
                .map_err(|_| anyhow!("SQID vector is of invalid length"))?;
            Ok(true)
        }
    }

    pub async fn send_sq(&self, answer: [&String; 3]) -> Result<()> {
        if answer[0].is_empty() || answer[1].is_empty() || answer[2].is_empty() {
            bail!("No answers for security questions provided");
        }
        let post_body = json!([
            {
                "id": self.sqid[0],
                "answer": answer[0],
            },
            {
                "id": self.sqid[1],
                "answer": answer[1],
            },
            {
                "id": self.sqid[2],
                "answer": answer[2]
            }
        ]);
        let res = self
            .client
            .post("https://api.mojang.com/user/security/location")
            .bearer_auth(&self.bearer_token)
            .json(&post_body)
            .send()
            .await?;
        let status = res.status();
        match status.as_u16() {
            200 => Ok(()),
            403 => bail!("Incorrect security questions"),
            _ => bail!(status),
        }
    }

    pub async fn check_name_availability_time(
        &self,
        username_to_snipe: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        let url = format!("https://api.star.shopping/droptime/{}", username_to_snipe);
        let res = self.client.get(url).send().await?;
        let status = res.status();
        let body = res.text().await?;
        let v: Value = serde_json::from_str(&body)?;
        match status.as_u16() {
            200 => {
                let epoch = v["unix"]
                    .as_i64()
                    .ok_or_else(|| anyhow!("Unable to parse `unix` from JSON"))?;
                let droptime = Utc.timestamp(epoch, 0);
                Ok(Some(droptime))
            }
            400 => {
                let error = v["error"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `error` from JSON"))?;
                let reason = if error == "username is not dropping" {
                    format!("{} is taken", username_to_snipe)
                } else if error == "username not dropping" {
                    format!("{} is available", username_to_snipe)
                } else {
                    error.to_string()
                };
                writeln!(
                    stdout(),
                    "{}",
                    Red.paint(format!("Failed to time snipe. Reason: {}", reason))
                )?;
                Ok(None)
            }
            _ => bail!(status),
        }
    }

    pub async fn check_name_change_eligibility(&self) -> Result<()> {
        let res = self
            .client
            .get("https://api.minecraftservices.com/minecraft/profile/namechange")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!(status);
        }
        let body = res.text().await?;
        let v: Value = serde_json::from_str(&body)?;
        let is_allowed = v["nameChangeAllowed"]
            .as_bool()
            .ok_or_else(|| anyhow!("Unable to parse `nameChangeAllowed` from JSON"))?;
        if !is_allowed {
            bail!("Name change not allowed within the cooldown period")
        }
        Ok(())
    }

    pub async fn upload_skin(&self, path: &str, skin_model: String) -> Result<()> {
        let img_file = File::open(path).await?;
        let stream = FramedRead::new(img_file, BytesCodec::new());
        let stream = Body::wrap_stream(stream);
        let image_part = reqwest::multipart::Part::stream(stream);
        let form = reqwest::multipart::Form::new()
            .text("variant", skin_model)
            .part("file", image_part);
        let res = self
            .client
            .post("https://api.minecraftservices.com/minecraft/profile/skins")
            .bearer_auth(&self.bearer_token)
            .multipart(form)
            .send()
            .await?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!(status);
        }
        Ok(())
    }

    pub async fn redeem_giftcode(&self, giftcode: &str) -> Result<()> {
        let url = format!(
            "https://api.minecraftservices.com/productvoucher/{}",
            giftcode
        );
        let res = self
            .client
            .put(url)
            .bearer_auth(&self.bearer_token)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!(status);
        }
        Ok(())
    }
}
