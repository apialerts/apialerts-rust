use serde_json::{json, Value};
use std::env;
use std::error::Error;
use ureq::{Agent, AgentBuilder};

pub struct ApiAlertsClient {
    api_key: String,
    agent: Agent,
}

impl ApiAlertsClient {
    pub fn new() -> Self {
        ApiAlertsClient {
            api_key: env::var("APIALERTS_API_KEY").unwrap_or_default(),
            agent: AgentBuilder::new().build(),
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    pub fn send(&self, message: &str, tags: &[String], link: &str) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("api key is missing".into());
        }
        if message.is_empty() {
            return Err("message is required".into());
        }

        let payload = json!({
            "message": message,
            "tags": tags,
            "link": link,
        });

        let url = "https://api.apialerts.com/event";

        let response = self
            .agent
            .post(url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .set("X-Integration", "rust")
            .set("X-Version", "1.0.0")
            .send_string(&payload.to_string())?;

        match response.status() {
            200 => {
                let data: Value = serde_json::from_reader(response.into_reader())?;
                println!(
                    "✓ (apialerts.com) Alert sent to {} successfully.",
                    data["project"]
                );
                Ok(())
            }
            400 => Err("bad request".into()),
            401 => Err("unauthorized".into()),
            403 => Err("forbidden".into()),
            429 => Err("rate limit exceeded".into()),
            _ => Err("unknown error".into()),
        }
    }
}

pub fn api_alerts_client() -> ApiAlertsClient {
    ApiAlertsClient::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = api_alerts_client();
        assert!(client.api_key.is_empty());
    }
}
