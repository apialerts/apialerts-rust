use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::event::Event;

const API_URL: &str = "https://api.apialerts.com/event";
const INTEGRATION_NAME: &str = "rust";
const INTEGRATION_VERSION: &str = "2.0.0";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// The result of an event delivery attempt.
///
/// `send_async` always returns this — check `success` rather than unwrapping.
#[derive(Debug, Clone)]
pub struct SendResult {
    pub success: bool,
    pub workspace: Option<String>,
    pub channel: Option<String>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

impl SendResult {
    fn ok(workspace: String, channel: String, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            workspace: Some(workspace),
            channel: Some(channel),
            warnings,
            error: None,
        }
    }

    pub(crate) fn from_error(message: impl Into<String>) -> Self {
        Self::err(message)
    }

    fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            workspace: None,
            channel: None,
            warnings: Vec::new(),
            error: Some(message.into()),
        }
    }
}

#[derive(Deserialize)]
struct ApiResponse {
    workspace: String,
    channel: String,
    #[serde(default)]
    warnings: Vec<String>,
}

/// An instance-based API Alerts client.
///
/// Use the free functions in `apialerts` (e.g. [`crate::configure`] / [`crate::send_async`])
/// for a convenient global singleton, or construct this type directly when you
/// need multiple independent clients or want full control over configuration.
pub struct ApiAlertsClient {
    api_key: String,
    integration: String,
    integration_version: String,
    base_url: String,
    debug: bool,
    http_client: Client,
}

impl ApiAlertsClient {
    /// Create a new client with the given API key and sensible defaults.
    pub fn new(api_key: impl Into<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("failed to build HTTP client");

        Self {
            api_key: api_key.into(),
            integration: INTEGRATION_NAME.to_string(),
            integration_version: INTEGRATION_VERSION.to_string(),
            base_url: API_URL.to_string(),
            debug: false,
            http_client,
        }
    }

    /// Enable or disable debug logging to stderr.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Override the integration name, version, and base URL.
    ///
    /// This is a first-party method used by official integrations (e.g. the
    /// GitHub Actions integration) and in tests to point the client at a mock
    /// server.
    pub fn set_overrides(
        mut self,
        integration: impl Into<String>,
        version: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        self.integration = integration.into();
        self.integration_version = version.into();
        self.base_url = base_url.into();
        self
    }

    /// Send an event — fire-and-forget. Never panics.
    ///
    /// Critical errors (not configured, missing key, empty message) are always
    /// logged to stderr. HTTP errors and successes are only logged when debug
    /// is enabled.
    pub async fn send(&self, event: Event) {
        // Critical checks — always log regardless of debug setting
        if self.api_key.is_empty() {
            eprintln!("x (apialerts.com) Error: api key is missing");
            return;
        }
        if event.message.is_empty() {
            eprintln!("x (apialerts.com) Error: message is required");
            return;
        }

        let result = self.post(&self.api_key, event).await;

        if self.debug {
            if !result.success {
                if let Some(ref msg) = result.error {
                    eprintln!("x (apialerts.com) Error: {}", msg);
                }
            } else {
                let workspace = result.workspace.as_deref().unwrap_or("");
                let channel = result.channel.as_deref().unwrap_or("");
                eprintln!("✓ (apialerts.com) Alert sent to {} ({})", workspace, channel);
                for w in &result.warnings {
                    eprintln!("! (apialerts.com) Warning: {}", w);
                }
            }
        }
    }

    /// Send an event and return the result. Never returns an error — check
    /// `result.success` instead.
    pub async fn send_async(&self, event: Event) -> SendResult {
        self.post(&self.api_key, event).await
    }

    /// Send an event using an explicit API key, bypassing the configured one.
    /// Never returns an error — check `result.success` instead.
    pub async fn send_with_key(
        &self,
        api_key: impl AsRef<str>,
        event: Event,
    ) -> SendResult {
        self.post(api_key.as_ref(), event).await
    }

    async fn post(&self, api_key: &str, event: Event) -> SendResult {
        if api_key.is_empty() {
            return SendResult::err("api key is missing");
        }
        if event.message.is_empty() {
            return SendResult::err("message is required");
        }

        let response = match self
            .http_client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("X-Integration", &self.integration)
            .header("X-Version", &self.integration_version)
            .json(&event)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return SendResult::err(e.to_string()),
        };

        match response.status().as_u16() {
            200 => {
                match response.json::<ApiResponse>().await {
                    Ok(body) => SendResult::ok(body.workspace, body.channel, body.warnings),
                    Err(_) => SendResult::err("invalid response from server"),
                }
            }
            400 => SendResult::err("bad request"),
            401 => SendResult::err("unauthorized — check your api key"),
            403 => SendResult::err("forbidden"),
            429 => SendResult::err("rate limit exceeded"),
            code => SendResult::err(format!("unexpected status: {}", code)),
        }
    }
}
