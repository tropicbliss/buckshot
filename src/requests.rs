use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, TimeZone, Utc};
use console::style;
use reqwest::{
    blocking::{multipart::Form, Client},
    header::ACCEPT,
};
use serde_json::{json, Value};
use std::{
    io::{stdout, Write},
    time::Duration,
};

pub struct Requests<'a> {
    client: Client,
    email: &'a str,
    password: &'a str,
}

impl<'a> Requests<'a> {
    pub fn new(email: &'a str, password: &'a str) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .user_agent("Sniper")
                .build()?,
            email,
            password,
        })
    }

    pub fn authenticate_mojang(&self) -> Result<String> {
        let post_json = json!({
            "username": self.email,
            "password": self.password
        });
        let res = self
            .client
            .post("https://authserver.mojang.com/authenticate")
            .json(&post_json)
            .send()?;
        let status = res.status();
        match status.as_u16() {
            200 => {
                let v: Value = serde_json::from_str(&res.text()?)?;
                let bearer_token = v["accessToken"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `accessToken` from JSON"))?
                    .to_string();
                Ok(bearer_token)
            }
            403 => {
                bail!("Incorrect email or password");
            }
            _ => {
                bail!("HTTP {}", status);
            }
        }
    }

    pub fn authenticate_microsoft(&self) -> Result<String> {
        let post_json = json!({
            "email": self.email,
            "password": self.password
        });
        let res = self
            .client
            .post("https://buckshot.tropicbliss.net/api/auth")
            .json(&post_json)
            .send()?;
        let status = res.status();
        match status.as_u16() {
            200 => {
                let body = res.text()?;
                let v: Value = serde_json::from_str(&body)?;
                let bearer_token = v["bearer_token"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `bearer_token` from JSON"))?
                    .to_string();
                Ok(bearer_token)
            }
            400 => {
                let body = res.text()?;
                let v: Value = serde_json::from_str(&body)?;
                let err = v["detail"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Unable to parse `detail` from JSON"))?;
                bail!("{}", err);
            }
            _ => {
                bail!("HTTP {}", status);
            }
        }
    }

    pub fn get_questions(&self, bearer_token: &str) -> Result<Option<[i64; 3]>> {
        let res = self
            .client
            .get("https://api.mojang.com/user/security/challenges")
            .bearer_auth(bearer_token)
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        let body = res.text()?;
        if body == "[]" {
            Ok(None)
        } else {
            let v: Value = serde_json::from_str(&body)?;
            let mut sqid_array = [0; 3];
            for idx in 0..2 {
                sqid_array[idx] = v[idx]["answer"]["id"].as_i64().ok_or_else(|| {
                    anyhow!(
                        "Unable to parse `answer` or `id` from index {} of JSON array",
                        idx
                    )
                })?;
            }
            Ok(Some(sqid_array))
        }
    }

    pub fn send_answers(
        &self,
        bearer_token: &str,
        questions: [i64; 3],
        answers: &[String; 3],
    ) -> Result<()> {
        let post_body = json!([
            {
                "id": questions[0],
                "answer": answers[0],
            },
            {
                "id": questions[1],
                "answer": answers[1],
            },
            {
                "id": questions[2],
                "answer": answers[2]
            }
        ]);
        let res = self
            .client
            .post("https://api.mojang.com/user/security/location")
            .bearer_auth(bearer_token)
            .json(&post_body)
            .send()?;
        let status = res.status();
        match status.as_u16() {
            200 => Ok(()),
            403 => bail!("Incorrect security questions"),
            _ => bail!("HTTP {}", status),
        }
    }

    pub fn check_name_availability_time(
        &self,
        username_to_snipe: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        let url = format!("https://api.star.shopping/droptime/{}", username_to_snipe);
        let res = self.client.get(url).send()?;
        let status = res.status();
        let body = res.text()?;
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
                    format!("{} is not taken", username_to_snipe)
                } else {
                    error.to_string()
                };
                writeln!(
                    stdout(),
                    "{}",
                    style(format!("Failed to get droptime: {}", reason)).red()
                )?;
                Ok(None)
            }
            _ => bail!("HTTP {}", status),
        }
    }

    pub fn check_name_change_eligibility(&self, bearer_token: &str) -> Result<()> {
        let res = self
            .client
            .get("https://api.minecraftservices.com/minecraft/profile/namechange")
            .bearer_auth(bearer_token)
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        let body = res.text()?;
        let v: Value = serde_json::from_str(&body)?;
        let is_allowed = v["nameChangeAllowed"]
            .as_bool()
            .ok_or_else(|| anyhow!("Unable to parse `nameChangeAllowed` from JSON"))?;
        if !is_allowed {
            bail!("Name change not allowed within the cooldown period")
        }
        Ok(())
    }

    pub fn upload_skin(&self, bearer_token: &str, path: &str, skin_model: String) -> Result<()> {
        let form = Form::new().text("variant", skin_model).file("file", path)?;
        let res = self
            .client
            .post("https://api.minecraftservices.com/minecraft/profile/skins")
            .bearer_auth(bearer_token)
            .multipart(form)
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        Ok(())
    }

    pub fn redeem_giftcode(&self, bearer_token: &str, giftcode: &str) -> Result<()> {
        let url = format!(
            "https://api.minecraftservices.com/productvoucher/{}",
            giftcode
        );
        let res = self
            .client
            .put(url)
            .bearer_auth(bearer_token)
            .header(ACCEPT, "application/json")
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        Ok(())
    }
}
