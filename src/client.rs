use std::fmt;
use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::event::Event;

const API_URL: &str = "https://api.apialerts.com/event";
const INTEGRATION_NAME: &str = "rust";
// Sourced from Cargo.toml so the X-Version header always matches the published crate.
const INTEGRATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const TIMEOUT_SECONDS: u64 = 30;

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
            Self::NotConfigured => write!(f, "client not configured"),
            Self::ApiKeyMissing => write!(f, "api key is missing"),
            Self::MessageRequired => write!(f, "message is required"),
            Self::NetworkError(msg) => write!(f, "{}", msg),
            Self::HttpError(_, msg) => write!(f, "{}", msg),
            Self::InvalidResponse => write!(f, "invalid response from server"),
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

/// An API Alerts client.
///
/// Construct it with [`ApiAlertsClient::new`] and reuse it for the lifetime of
/// your app. The Rust SDK is instance-based by design; there is no global
/// singleton.
#[derive(Clone)]
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
            .timeout(Duration::from_secs(TIMEOUT_SECONDS))
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
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Override the `X-Integration` / `X-Version` headers and the base URL.
    ///
    /// Internal hook for first-party integrations (e.g. GitHub Actions) and tests.
    pub fn set_overrides(
        &mut self,
        integration: impl Into<String>,
        version: impl Into<String>,
        base_url: impl Into<String>,
    ) {
        self.integration = integration.into();
        self.integration_version = version.into();
        self.base_url = base_url.into();
    }

    /// Send an event - fire-and-forget using the configured API key. Never panics.
    pub async fn send(&self, event: Event) {
        self.send_internal(&self.api_key.clone(), event).await;
    }

    /// Send an event - fire-and-forget using an explicit API key. Never panics.
    pub async fn send_with_key(&self, api_key: impl AsRef<str>, event: Event) {
        self.send_internal(api_key.as_ref(), event).await;
    }

    async fn send_internal(&self, key: &str, event: Event) {
        // Critical checks - always log regardless of debug setting
        if key.is_empty() {
            eprintln!("x (apialerts.com) Error: api key is missing");
            return;
        }
        if event.message.is_empty() {
            eprintln!("x (apialerts.com) Error: message is required");
            return;
        }

        let result = self.post(key, event).await;

        if self.debug {
            match result {
                Ok(r) => {
                    eprintln!(
                        "✓ (apialerts.com) Alert sent to {} ({})",
                        r.workspace, r.channel
                    );
                    for w in &r.warnings {
                        eprintln!("! (apialerts.com) Warning: {}", w);
                    }
                }
                Err(e) => eprintln!("x (apialerts.com) Error: {}", e),
            }
        }
    }

    /// Send an event and return the result using the configured API key.
    pub async fn send_async(&self, event: Event) -> Result<SendResult, ApiAlertsError> {
        self.post(&self.api_key.clone(), event).await
    }

    /// Send an event and return the result using an explicit API key.
    pub async fn send_async_with_key(
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
            401 => Err(ApiAlertsError::HttpError(
                401,
                "unauthorized - check your api key".into(),
            )),
            403 => Err(ApiAlertsError::HttpError(403, "forbidden".into())),
            429 => Err(ApiAlertsError::HttpError(429, "rate limit exceeded".into())),
            code => Err(ApiAlertsError::HttpError(
                code,
                format!("unexpected status: {}", code),
            )),
        }
    }
}

#[cfg(test)]
mod constants_tests {
    use super::*;

    #[test]
    fn integration_name_is_rust() {
        assert_eq!(INTEGRATION_NAME, "rust");
    }

    #[test]
    fn base_url_is_production_event_endpoint() {
        assert_eq!(API_URL, "https://api.apialerts.com/event");
    }

    #[test]
    fn timeout_is_30_seconds() {
        assert_eq!(TIMEOUT_SECONDS, 30);
    }

    #[test]
    fn version_is_a_1_x_release() {
        // Pins the major version at 1.x and guards against an accidental 2.0 bump.
        let parts: Vec<&str> = INTEGRATION_VERSION.split(['.', '-', '+']).collect();
        assert_eq!(
            parts[0], "1",
            "expected a 1.x version, got {INTEGRATION_VERSION}"
        );
        assert!(
            parts.len() >= 3,
            "expected semver x.y.z, got {INTEGRATION_VERSION}"
        );
        assert!(
            parts[1].parse::<u32>().is_ok(),
            "minor not numeric: {INTEGRATION_VERSION}"
        );
        assert!(
            parts[2].parse::<u32>().is_ok(),
            "patch not numeric: {INTEGRATION_VERSION}"
        );
    }
}
