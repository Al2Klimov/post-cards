use crate::store::Credentials;
use percent_encoding_rfc3986::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use serde_json::json;
use std::time::SystemTime;
use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";
const SCOPES: &str = "read write";

pub struct Client {
    agent: Agent,
    base: String,
    token: Option<String>,
}

#[derive(Deserialize)]
struct App {
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
}

#[derive(Deserialize)]
struct Account {
    id: String,
}

#[derive(Deserialize)]
struct Status {
    created_at: String,
}

#[derive(Deserialize)]
struct Scheduled {
    scheduled_at: String,
}

pub fn normalize_domain(s: &str) -> String {
    s.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

pub fn parse_time(s: &str) -> Result<SystemTime, String> {
    let s = s
        .strip_suffix("+00:00")
        .map_or(s.to_string(), |x| format!("{x}Z"));
    humantime::parse_rfc3339(&s).map_err(|e| format!("bad timestamp {s}: {e}"))
}

impl Client {
    pub fn new(domain: &str, token: Option<String>) -> Self {
        let agent: Agent = Agent::config_builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    .build(),
            )
            .user_agent(concat!("post_cards/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        Client {
            agent,
            base: format!("https://{domain}"),
            token,
        }
    }

    fn auth(&self) -> String {
        ["Bearer", self.token.as_deref().unwrap_or_default()].join(" ")
    }

    pub fn register(&self, domain: &str) -> Result<Credentials, String> {
        let app: App = self
            .agent
            .post(format!("{}/api/v1/apps", self.base))
            .send_json(json!({
                "client_name": "post_cards",
                "redirect_uris": REDIRECT_URI,
                "scopes": SCOPES,
            }))
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;
        Ok(Credentials {
            domain: domain.to_string(),
            client_id: app.client_id,
            client_secret: app.client_secret,
            access_token: None,
        })
    }

    pub fn authorize_url(&self, c: &Credentials) -> String {
        let enc = |s: &str| utf8_percent_encode(s, NON_ALPHANUMERIC).to_string();
        format!(
            "{}/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
            self.base,
            enc(&c.client_id),
            enc(REDIRECT_URI),
            enc(SCOPES)
        )
    }

    pub fn exchange_code(&self, c: &Credentials, code: &str) -> Result<String, String> {
        let t: Token = self
            .agent
            .post(format!("{}/oauth/token", self.base))
            .send_json(json!({
                "grant_type": "authorization_code",
                "client_id": c.client_id,
                "client_secret": c.client_secret,
                "redirect_uri": REDIRECT_URI,
                "scope": SCOPES,
                "code": code,
            }))
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;
        Ok(t.access_token)
    }

    fn account_id(&self) -> Result<String, String> {
        let a: Account = self
            .agent
            .get(format!("{}/api/v1/accounts/verify_credentials", self.base))
            .header("Authorization", self.auth())
            .call()
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;
        Ok(a.id)
    }

    pub fn verify(&self) -> Result<(), String> {
        self.account_id().map(|_| ())
    }

    /// Latest of the newest published post and all scheduled posts,
    /// plus the number of scheduled (not yet published) posts.
    pub fn queue(&self) -> Result<(Option<SystemTime>, usize), String> {
        let id = self.account_id()?;
        let statuses: Vec<Status> = self
            .agent
            .get(format!(
                "{}/api/v1/accounts/{id}/statuses?limit=1",
                self.base
            ))
            .header("Authorization", self.auth())
            .call()
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;
        let scheduled: Vec<Scheduled> = self
            .agent
            .get(format!("{}/api/v1/scheduled_statuses?limit=80", self.base))
            .header("Authorization", self.auth())
            .call()
            .map_err(|e| e.to_string())?
            .body_mut()
            .read_json()
            .map_err(|e| e.to_string())?;
        let times = statuses
            .iter()
            .map(|s| &s.created_at)
            .chain(scheduled.iter().map(|s| &s.scheduled_at));
        let mut max = None;
        for t in times {
            let t = parse_time(t)?;
            max = max.max(Some(t));
        }
        Ok((max, scheduled.len()))
    }

    pub fn post(&self, text: &str, at: Option<SystemTime>) -> Result<(), String> {
        let mut body = json!({"status": text, "visibility": "public", "language": "en"});
        if let Some(at) = at {
            body["scheduled_at"] = humantime::format_rfc3339_millis(at).to_string().into();
        }
        self.agent
            .post(format!("{}/api/v1/statuses", self.base))
            .header("Authorization", self.auth())
            .send_json(body)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain() {
        assert_eq!(
            normalize_domain(" https://social.example.org/ "),
            "social.example.org"
        );
    }

    #[test]
    fn time() {
        assert_eq!(
            parse_time("2026-10-04T15:21:30.145+00:00").unwrap(),
            parse_time("2026-10-04T15:21:30.145Z").unwrap()
        );
    }
}
