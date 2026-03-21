use std::fmt;
use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::event::Event;

const API_URL: &str = "https://api.apialerts.com/event";
const INTEGRATION_NAME: &str = "rust";
const INTEGRATION_VERSION: &str = "2.0.0";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// The result of a successful event delivery.
#[derive(Debug, Clone)]
pub struct SendResult {
    pub workspace: String,
    pub channel: String,
    pub warnings: Vec<String>,
}

/// Errors that can be returned by the API Alerts SDK.
#[derive(Debug)]
pub enum ApiAlertsError {
    NotConfigured,
    ApiKeyMissing,
    MessageRequired,
    NetworkError(String),
    HttpError(u16, String),
    InvalidResponse,
}

impl fmt::Display for ApiAlertsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured       => write!(f, "client not configured"),
            Self::ApiKeyMissing       => write!(f, "api key is missing"),
            Self::MessageRequired     => write!(f, "message is required"),
            Self::NetworkError(msg)   => write!(f, "{}", msg),
            Self::HttpError(_, msg)   => write!(f, "{}", msg),
            Self::InvalidResponse     => write!(f, "invalid response from server"),
        }
    }
}

impl std::error::Error for ApiAlertsError {}

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
            match result {
                Ok(r) => {
                    eprintln!("✓ (apialerts.com) Alert sent to {} ({})", r.workspace, r.channel);
                    for w in &r.warnings {
                        eprintln!("! (apialerts.com) Warning: {}", w);
                    }
                }
                Err(e) => eprintln!("x (apialerts.com) Error: {}", e),
            }
        }
    }

    /// Send an event and return the result.
    pub async fn send_async(&self, event: Event) -> Result<SendResult, ApiAlertsError> {
        self.post(&self.api_key, event).await
    }

    /// Send an event using an explicit API key, bypassing the configured one.
    pub async fn send_with_key(
        &self,
        api_key: impl AsRef<str>,
        event: Event,
    ) -> Result<SendResult, ApiAlertsError> {
        self.post(api_key.as_ref(), event).await
    }

    async fn post(&self, api_key: &str, event: Event) -> Result<SendResult, ApiAlertsError> {
        if api_key.is_empty() {
            return Err(ApiAlertsError::ApiKeyMissing);
        }
        if event.message.is_empty() {
            return Err(ApiAlertsError::MessageRequired);
        }

        let response = self
            .http_client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("X-Integration", &self.integration)
            .header("X-Version", &self.integration_version)
            .json(&event)
            .send()
            .await
            .map_err(|e| ApiAlertsError::NetworkError(e.to_string()))?;

        match response.status().as_u16() {
            200 => response
                .json::<ApiResponse>()
                .await
                .map(|body| SendResult {
                    workspace: body.workspace,
                    channel: body.channel,
                    warnings: body.warnings,
                })
                .map_err(|_| ApiAlertsError::InvalidResponse),
            400 => Err(ApiAlertsError::HttpError(400, "bad request".into())),
            401 => Err(ApiAlertsError::HttpError(401, "unauthorized — check your api key".into())),
            403 => Err(ApiAlertsError::HttpError(403, "forbidden".into())),
            429 => Err(ApiAlertsError::HttpError(429, "rate limit exceeded".into())),
            code => Err(ApiAlertsError::HttpError(code, format!("unexpected status: {}", code))),
        }
    }
}
