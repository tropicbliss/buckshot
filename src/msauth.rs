use anyhow::{anyhow, bail, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{blocking::Client, header::ACCEPT};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, time::Duration};

pub struct Auth<'a> {
    client: Client,
    email: &'a str,
    password: &'a str,
}

struct LoginData {
    ppft: String,
    url_post: String,
}

#[derive(Deserialize)]
struct AuthData {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: [Uhs; 1],
}

#[derive(Deserialize)]
struct Uhs {
    uhs: String,
}

#[derive(Deserialize)]
struct FailedAuthData {
    #[serde(rename = "XErr")]
    x_err: u64,
}

#[derive(Deserialize)]
struct BearerToken {
    access_token: String,
}

impl<'a> Auth<'a> {
    pub fn new(email: &'a str, password: &'a str) -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self {
            client,
            email,
            password,
        })
    }

    pub fn authenticate(&self) -> Result<String> {
        let access_token = self
            .get_access_token()
            .with_context(|| "Unable to get access token")?;
        let bearer_token = self
            .get_bearer_token(&access_token)
            .with_context(|| "Unable to get bearer token")?;
        Ok(bearer_token)
    }

    fn get_access_token(&self) -> Result<String> {
        let login_data = self
            .get_login_data()
            .with_context(|| "Unable to get login data")?;
        let access_token = self
            .sign_in(&login_data)
            .with_context(|| "Unable to get access token")?;
        Ok(access_token)
    }

    fn get_login_data(&self) -> Result<LoginData> {
        lazy_static! {
            static ref PPFT_RE: Regex = Regex::new(r#"value="(.+?)""#).unwrap();
            static ref URLPOST_RE: Regex = Regex::new("urlPost:'(.+?)'").unwrap();
        }
        let res = self.client.get("https://login.live.com/oauth20_authorize.srf?client_id=000000004C12AE6F&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=service::user.auth.xboxlive.com::MBI_SSL&display=touch&response_type=token&locale=en").send()?;
        let html = res.text()?;
        let ppft_captures = PPFT_RE
            .captures(&html)
            .ok_or_else(|| anyhow!("Unable to capture PPFT from regex"))?;
        let ppft = ppft_captures
            .get(1)
            .ok_or_else(|| anyhow!("Unable to get PPFT"))?
            .as_str()
            .to_string();
        let urlpost_captures = URLPOST_RE
            .captures(&html)
            .with_context(|| anyhow!("Unable to capture POST URL from regex"))?;
        let url_post = urlpost_captures
            .get(1)
            .ok_or_else(|| anyhow!("Unable to get POST URL"))?
            .as_str()
            .to_string();
        Ok(LoginData { ppft, url_post })
    }

    fn sign_in(&self, login_data: &LoginData) -> Result<String> {
        let params = [
            ("login", self.email),
            ("loginfmt", self.email),
            ("passwd", self.password),
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
            .ok_or_else(|| anyhow!(r#"Please dismiss "unusual account activity" alert at https://account.live.com/activity"#))?
            .split('&')
            .map(|kv| {
                let mut key_value: Vec<&str> = kv.split('=').collect();
                (key_value.remove(0), key_value.remove(0))
            })
            .collect();
        Ok(param
            .remove("access_token")
            .ok_or_else(|| anyhow!("Unable to extract access_token from redirect URL"))?
            .to_string())
    }

    fn get_bearer_token(&self, access_token: &str) -> Result<String> {
        let xbl_data = self
            .authenticate_with_xbl(access_token)
            .with_context(|| "Unable to get Xbox Live data")?;
        let xsts_token = self
            .authenticate_with_xsts(&xbl_data.token)
            .with_context(|| "Unable to get XSTS token")?;
        let bearer_token = self
            .authenticate_with_minecraft(&xbl_data.display_claims.xui[0].uhs, &xsts_token)
            .with_context(|| "Unable to get bearer token")?;
        Ok(bearer_token)
    }

    fn authenticate_with_xbl(&self, access_token: &str) -> Result<AuthData> {
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
        let auth_data: AuthData = serde_json::from_str(&res.text()?)?;
        Ok(auth_data)
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
        match status.as_u16() {
            401 => {
                let err: FailedAuthData = serde_json::from_str(&text)?;
                if err.x_err == 2_148_916_233 {
                    bail!("This account doesn't have an Xbox account");
                }
                if err.x_err == 2_148_916_238 {
                    bail!("The account is a child (under 18) and cannot proceed unless the account is added to a family by an adult");
                }
                bail!("Something went wrong: XErr: {}", err.x_err);
            }
            200 => {
                let auth_data: AuthData = serde_json::from_str(&text)?;
                Ok(auth_data.token)
            }
            _ => {
                bail!("HTTP {}", status);
            }
        }
    }

    fn authenticate_with_minecraft(&self, userhash: &str, xsts_token: &str) -> Result<String> {
        let json = json!({ "identityToken": format!("XBL3.0 x={};{}", userhash, xsts_token) });
        let res = self
            .client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&json)
            .send()?;
        let status = res.status();
        if status.as_u16() != 200 {
            bail!("HTTP {}", status);
        }
        let bearer_token: BearerToken = serde_json::from_str(&res.text()?)?;
        Ok(bearer_token.access_token)
    }
}
