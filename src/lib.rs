pub mod config;
pub mod event;

use std::error::Error;

use async_std::task::block_on;
use config::ApiAlertsConfig;
use event::ApiAlertsEvent;
use reqwest::{Client, StatusCode};

pub struct ApiAlertsClient {
    api_key: String,
    config: ApiAlertsConfig,
}

pub const API_URL: &str = "https://api.apialerts.com/event";
pub const X_INTEGRATION: &str = "rust";
pub const X_VERSION: &str = "1.0.0";

impl ApiAlertsClient {
    pub fn new(api_key: String) -> Self {
        ApiAlertsClient {
            api_key,
            config: ApiAlertsConfig::new_default_config(),
        }
    }

    pub fn update_config(mut self, config: ApiAlertsConfig) -> Self {
        self.config = config;
        self
    }

    pub fn update_api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self
    }

    pub async fn send_async_with_api_key(
        &self,
        api_key: String,
        event: ApiAlertsEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.api_key.is_empty() {
            return Err("api key is missing".into());
        }

        if event.message.is_empty() {
            return Err("message is required".into());
        }

        let client = Client::new();

        let payload = event.convert_to_json();

        let response = client
            .post(API_URL)
            .header("Authorization", &format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("X-Integration", X_INTEGRATION)
            .header("X-Version", X_VERSION)
            .json(&payload)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let data: serde_json::Value = response.json().await?;
                if self.config.logging {
                    println!(
                        "✓ (apialerts.com) Alert sent to {} successfully.",
                        data["project"]
                    );
                }
                Ok(())
            }
            StatusCode::BAD_REQUEST => Err("bad request".into()),
            StatusCode::UNAUTHORIZED => Err("unauthorized".into()),
            StatusCode::FORBIDDEN => Err("forbidden".into()),
            StatusCode::TOO_MANY_REQUESTS => Err("rate limit exceeded".into()),
            _ => Err("unknown error".into()),
        }
    }

    pub fn send_with_api_key(
        &self,
        api_key: String,
        event: ApiAlertsEvent,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        block_on(self.send_async_with_api_key(api_key, event))
    }

    pub fn send(&self, event: ApiAlertsEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        block_on(self.send_async_with_api_key(self.api_key.clone(), event))
    }

    pub async fn send_async(
        &self,
        event: ApiAlertsEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send_async_with_api_key(self.api_key.clone(), event)
            .await
    }
}
