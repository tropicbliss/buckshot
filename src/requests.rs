use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use console::style;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    blocking::{multipart::Form, Client},
    header::ACCEPT,
};
use serde_json::{json, Value};
use std::{collections::HashMap, path::Path, time::Duration};

pub struct Requests {
    client: Client,
}

struct LoginData {
    ppft: String,
    url_post: String,
}

struct XBLData {
    token: String,
    userhash: String,
}

impl Requests {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .user_agent("Sniper")
                .cookie_store(true)
                .build()?,
        })
    }

    pub fn authenticate_mojang(&self, email: &str, password: &str) -> Result<String> {
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

    pub fn authenticate_microsoft(&self, email: &str, password: &str) -> Result<String> {
        let access_token = self
            .get_access_token(email, password)
            .with_context(|| "Error getting access token")?;
        let bearer_token = self
            .get_bearer_token(&access_token)
            .with_context(|| "Error getting bearer token")?;
        Ok(bearer_token)
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

    pub fn check_name_availability_time(&self, name: &str) -> Result<Option<DateTime<Utc>>> {
        let url = format!("https://api.star.shopping/droptime/{}", name);
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
                println!(
                    "{}",
                    style(format!("Failed to get droptime of {}: {}", name, error)).red()
                );
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

    pub fn upload_skin(
        &self,
        bearer_token: &str,
        skin_path: &Path,
        skin_model: String,
    ) -> Result<()> {
        let form = Form::new()
            .text("variant", skin_model)
            .file("file", skin_path)?;
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

    fn get_access_token(&self, email: &str, password: &str) -> Result<String> {
        let login_data = self
            .get_login_data()
            .with_context(|| "Error getting login data")?;
        let access_token = self
            .sign_in(email, password, &login_data)
            .with_context(|| "Error getting access token")?;
        Ok(access_token)
    }

    fn get_login_data(&self) -> Result<LoginData> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"value="(.+?)""#).unwrap();
        }
        let res = self.client.get("https://login.live.com/oauth20_authorize.srf?client_id=000000004C12AE6F&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=service::user.auth.xboxlive.com::MBI_SSL&display=touch&response_type=token&locale=en").send()?;
        let html = res.text()?;
        let ppft_captures = RE
            .captures(&html)
            .ok_or_else(|| anyhow!("Error capturing PPFT from regex"))?;
        let ppft = ppft_captures
            .get(1)
            .ok_or_else(|| anyhow!("Error getting PPFT"))?
            .as_str()
            .to_string();
        let urlpost_re = Regex::new(r#"urlPost:'(.+?)'"#)?;
        let urlpost_captures = urlpost_re
            .captures(&html)
            .with_context(|| anyhow!("Error capturing POST URL from regex"))?;
        let url_post = urlpost_captures
            .get(1)
            .ok_or_else(|| anyhow!("Error getting POST URL"))?
            .as_str()
            .to_string();
        Ok(LoginData { ppft, url_post })
    }

    fn sign_in(&self, email: &str, password: &str, login_data: &LoginData) -> Result<String> {
        let params = [
            ("login", email),
            ("loginfmt", email),
            ("passwd", password),
            ("PPFT", &login_data.ppft),
        ];
        let res = self
            .client
            .post(&login_data.url_post)
            .form(&params)
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        let url = res.url().clone();
        let text = res.text()?;
        if !url.to_string().contains("access_token") && url.as_str() == login_data.url_post {
            if text.contains("Sign in to") {
                bail!("Incorrect credentials");
            }
            if text.contains("2FA is enabled but not supported yet!") {
                bail!("Please disable 2FA at https://account.live.com/activity");
            }
        }
        let mut param: HashMap<&str, &str> = url
            .fragment()
            .ok_or_else(|| anyhow!("Error parsing params"))?
            .split('&')
            .map(|kv| {
                let mut key_value: Vec<&str> = kv.split('=').collect();
                (key_value.remove(0), key_value.remove(0))
            })
            .collect();
        Ok(param
            .remove("access_token")
            .ok_or_else(|| anyhow!("Error getting access token from params"))?
            .to_string())
    }

    fn get_bearer_token(&self, access_token: &str) -> Result<String> {
        let xbl_data = self
            .authenticate_with_xbl(access_token)
            .with_context(|| "Error getting Xbox Live data")?;
        let xsts_token = self
            .authenticate_with_xsts(&xbl_data.token)
            .with_context(|| "Error getting XSTS token")?;
        let bearer_token = self
            .authenticate_with_minecraft(&xbl_data.userhash, &xsts_token)
            .with_context(|| "Error getting bearer token")?;
        Ok(bearer_token)
    }

    fn authenticate_with_xbl(&self, access_token: &str) -> Result<XBLData> {
        let json = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": access_token
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });
        let res = self
            .client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&json)
            .header(ACCEPT, "application/json")
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        let text = res.text()?;
        let v: Value = serde_json::from_str(&text)?;
        let token = v["Token"]
            .as_str()
            .ok_or_else(|| anyhow!("Error parsing access token from JSON"))?
            .to_string();
        let userhash = v["DisplayClaims"]["xui"][0]["uhs"]
            .as_str()
            .ok_or_else(|| anyhow!("Error parsing user hash from JSON"))?
            .to_string();
        Ok(XBLData { token, userhash })
    }

    fn authenticate_with_xsts(&self, token: &str) -> Result<String> {
        let json = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });
        let res = self
            .client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header(ACCEPT, "application/json")
            .json(&json)
            .send()?;
        let status = res.status();
        let text = res.text()?;
        let v: Value = serde_json::from_str(&text)?;
        if status.as_u16() == 401 {
            let err = v["XErr"]
                .as_u64()
                .ok_or_else(|| anyhow!("Error parsing error message from JSON"))?;
            if err == 2_148_916_233 {
                bail!("The account doesn't have an Xbox account. Once they sign up for one (or login through minecraft.net to create one) then they can proceed with the login. This shouldn't happen with accounts that have purchased Minecraft with a Microsoft account, as they would've already gone through that Xbox signup process.");
            }
            if err == 2_148_916_238 {
                bail!("The account is a child (under 18) and cannot proceed unless the account is added to a Family by an adult. This only seems to occur when using a custom Microsoft Azure application. When using the Minecraft launchers client id, this doesn't trigger.");
            }
            bail!("Something went wrong.");
        } else if status.as_u16() == 200 {
            let token = v["Token"]
                .as_str()
                .ok_or_else(|| anyhow!("Error parsing XSTS token from JSON"))?
                .to_string();
            Ok(token)
        } else {
            bail!("HTTP {}", status);
        }
    }

    fn authenticate_with_minecraft(&self, userhash: &str, xsts_token: &str) -> Result<String> {
        let json = json!({ "identityToken": format!("XBL3.0 x={};{}", userhash, xsts_token) });
        let res = self.client.post("https://api.minecraftservices.com/authentication/login_with_xbox").json(&json).send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        let text = res.text()?;
        let v: Value = serde_json::from_str(&text)?;
        let bearer = v["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("Error parsing bearer token from JSON"))?
            .to_string();
        Ok(bearer)
    }
}
