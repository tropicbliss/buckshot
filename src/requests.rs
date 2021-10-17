use anyhow::{bail, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use reqwest::blocking::{multipart::Form, Client};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

pub struct Requests {
    client: Client,
}

pub enum DroptimeData {
    Available(DateTime<Utc>),
    Unavailable(String),
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct BearerToken {
    accessToken: String,
}

#[derive(Deserialize)]
struct AvailableDroptime {
    unix: i64,
}

#[derive(Deserialize)]
struct UnavailableDroptime {
    error: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct NameChangeEligibility {
    nameChangeAllowed: bool,
}

#[derive(Deserialize)]
pub struct QuestionData {
    answer: QuestionID,
}

#[derive(Deserialize)]
pub struct QuestionID {
    id: u64,
}

impl Requests {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .user_agent("Sniper")
                .build()?,
        })
    }

    pub fn authenticate_mojang(
        &self,
        email: &str,
        password: &str,
        answers: &Option<[String; 3]>,
    ) -> Result<String> {
        let bearer_token = self
            .get_bearer_token(email, password)
            .with_context(|| "Error getting bearer token")?;
        if let Some(questions) = self
            .get_questions(&bearer_token)
            .with_context(|| format!("Failed to get SQ IDs"))?
        {
            match answers {
                Some(x) => {
                    self.send_answers(&bearer_token, questions, x)
                        .with_context(|| format!("Failed to send SQ answers"))?;
                }
                None => {
                    bail!("SQ answers required");
                }
            }
        }
        Ok(bearer_token)
    }

    fn get_bearer_token(&self, email: &str, password: &str) -> Result<String> {
        let post_json = json!({
            "username": email,
            "password": password
        });
        let res = self
            .client
            .post("https://authserver.mojang.com/authenticate")
            .json(&post_json)
            .send()?;
        let status = res.status();
        match status.as_u16() {
            200 => {
                let bearer_token: BearerToken = serde_json::from_str(&res.text()?)?;
                Ok(bearer_token.accessToken)
            }
            403 => {
                bail!("Incorrect email or password");
            }
            _ => {
                bail!("HTTP {}", status);
            }
        }
    }

    fn get_questions(&self, bearer_token: &str) -> Result<Option<[QuestionData; 3]>> {
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
            let sqid_array: [QuestionData; 3] = serde_json::from_str(&body)?;
            Ok(Some(sqid_array))
        }
    }

    fn send_answers(
        &self,
        bearer_token: &str,
        questions: [QuestionData; 3],
        answers: &[String; 3],
    ) -> Result<()> {
        let post_body = json!([
            {
                "id": questions[0].answer.id,
                "answer": answers[0],
            },
            {
                "id": questions[1].answer.id,
                "answer": answers[1],
            },
            {
                "id": questions[2].answer.id,
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
            204 => Ok(()),
            403 => bail!("Incorrect security questions"),
            _ => bail!("HTTP {}", status),
        }
    }

    pub fn check_name_availability_time(&self, name: &str) -> Result<DroptimeData> {
        let url = format!("http://api.star.shopping/droptime/{}", name);
        let res = self.client.get(url).send()?;
        let status = res.status();
        let body = res.text()?;
        match status.as_u16() {
            200 => {
                let epoch: AvailableDroptime = serde_json::from_str(&body)?;
                let droptime = Utc.timestamp(epoch.unix, 0);
                Ok(DroptimeData::Available(droptime))
            }
            400 => {
                let error: UnavailableDroptime = serde_json::from_str(&body)?;
                Ok(DroptimeData::Unavailable(error.error))
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
        let is_allowed: NameChangeEligibility = serde_json::from_str(&body)?;
        if !is_allowed.nameChangeAllowed {
            bail!("Name change not allowed within the cooldown period")
        }
        Ok(())
    }

    pub fn upload_skin(
        &self,
        bearer_token: &str,
        path: &str,
        skin_model: String,
        is_file: bool,
    ) -> Result<()> {
        let res = self
            .client
            .post("https://api.minecraftservices.com/minecraft/profile/skins")
            .bearer_auth(bearer_token);
        let res = if is_file {
            let form = Form::new().text("variant", skin_model).file("file", path)?;
            res.multipart(form)
        } else {
            let post_body = json!({
                "url": path,
                "variant": skin_model
            });
            res.json(&post_body)
        };
        let res = res.send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        Ok(())
    }
}
