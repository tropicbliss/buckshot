use crate::constants;
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::{Body, Client};
use serde_json::{json, Value};
use std::{convert::TryInto, time::Duration};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

pub struct Requests {
    client: Client,
}

impl Requests {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .tcp_keepalive(Some(Duration::from_secs(5)))
                .use_rustls_tls()
                .build()?,
        })
    }

    pub async fn authenticate_mojang(&self, username: &str, password: &str) -> Result<String> {
        if username.is_empty() || password.is_empty() {
            bail!("No email or password provided");
        }
        let post_json = json!({
            "username": username,
            "password": password
        });
        let url = format!("{}/authenticate", constants::YGGDRASIL_ORIGIN_SERVER);
        let res = self.client.post(url).json(&post_json).send().await?;
        match res.status().as_u16() {
            200 => {
                let v: Value = serde_json::from_str(&res.text().await?)?;
                let access_token = v["accessToken"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `accessToken` from JSON"))?
                    .to_string();
                Ok(access_token)
            }
            403 => bail!("Incorrect email or password"),
            status => bail!("HTTP {}", status),
        }
    }

    pub async fn authenticate_microsoft(&self, username: &str, password: &str) -> Result<String> {
        if username.is_empty() || password.is_empty() {
            bail!("No email or password provided");
        }
        let post_json = json!({
            "username": username,
            "password": password
        });
        let url = format!("{}/simpleauth", constants::BUCKSHOT_API_SERVER);
        let res = self.client.post(url).json(&post_json).send().await?;
        match res.status().as_u16() {
            200 => {
                let body = res.text().await?;
                let v: Value = serde_json::from_str(&body)?;
                Ok(v["access_token"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `access_token` from JSON"))?
                    .to_string())
            }
            400 => {
                let body = res.text().await?;
                let v: Value = serde_json::from_str(&body)?;
                let err = v["error"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `error` from JSON"))?
                    .to_string();
                bail!("{}", err);
            }
            status => bail!("HTTP {}", status),
        }
    }

    pub async fn get_sq_id(&self, access_token: &str) -> Result<Option<[i64; 3]>> {
        let url = format!("{}/user/security/challenges", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await?;
        let status = res.status().as_u16();
        if status != 200 {
            bail!("HTTP {}", status);
        }
        let body = res.text().await?;
        if body == "[]" {
            Ok(None)
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
            let sqid_array = sqid_array
                .try_into()
                .map_err(|_| anyhow!("SQID vector is of invalid length"))?;
            Ok(Some(sqid_array))
        }
    }

    pub async fn send_sq(
        &self,
        access_token: &str,
        id: [i64; 3],
        answer: [&String; 3],
    ) -> Result<()> {
        if answer[0].is_empty() || answer[1].is_empty() || answer[2].is_empty() {
            bail!("No answers for security questions provided");
        }
        let post_body = json!([
            {
                "id": id[0],
                "answer": answer[0],
            },
            {
                "id": id[1],
                "answer": answer[1],
            },
            {
                "id": id[2],
                "answer": answer[2]
            }
        ]);
        let url = format!("{}/user/security/location", constants::MOJANG_API_SERVER);
        let res = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .json(&post_body)
            .send()
            .await?;
        match res.status().as_u16() {
            204 => Ok(()),
            403 => bail!("Incorrect security questions"),
            status => bail!("HTTP {}", status),
        }
    }

    pub async fn check_name_availability_time(
        &self,
        username_to_snipe: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        let url = format!("{}/droptime/{}", constants::NAMEMC_API, username_to_snipe);
        let res = self.client.get(url).send().await?;
        let status = res.status().as_u16();
        match status {
            200 => {
                let body = res.text().await?;
                let v: Value = serde_json::from_str(&body)?;
                let epoch = v["UNIX"]
                    .as_i64()
                    .ok_or_else(|| anyhow!("Unable to parse `UNIX` from JSON"))?;
                let droptime = Utc.timestamp(epoch, 0);
                Ok(Some(droptime))
            }
            404 => Ok(None),
            status => {
                bail!("HTTP {}", status);
            }
        }
    }

    pub async fn check_name_change_eligibility(&self, access_token: &str) -> Result<()> {
        let url = format!(
            "{}/minecraft/profile/namechange",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await?;
        let status = res.status().as_u16();
        if status != 200 {
            bail!("HTTP {}", status);
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

    pub async fn upload_skin(
        &self,
        path: &str,
        skin_model: String,
        access_token: &str,
    ) -> Result<()> {
        let img_file = File::open(path).await?;
        let stream = FramedRead::new(img_file, BytesCodec::new());
        let stream = Body::wrap_stream(stream);
        let image_part = reqwest::multipart::Part::stream(stream);
        let form = reqwest::multipart::Form::new()
            .text("variant", skin_model)
            .part("file", image_part);
        let url = format!(
            "{}/minecraft/profile/skins",
            constants::MINECRAFTSERVICES_API_SERVER
        );
        let res = self
            .client
            .post(url)
            .bearer_auth(access_token)
            .multipart(form)
            .send()
            .await?;
        let status = res.status().as_u16();
        if status != 200 {
            bail!("HTTP {}", status);
        }
        Ok(())
    }

    pub async fn redeem_giftcode(&self, giftcode: &str, access_token: &str) -> Result<()> {
        let url = format!(
            "{}/productvoucher/{}",
            constants::MINECRAFTSERVICES_API_SERVER,
            giftcode
        );
        let res = self
            .client
            .put(url)
            .bearer_auth(access_token)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;
        let status = res.status().as_u16();
        if status != 200 {
            bail!("HTTP {}", status);
        }
        Ok(())
    }
}
