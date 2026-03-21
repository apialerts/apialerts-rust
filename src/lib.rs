mod client;
mod event;

pub use client::{ApiAlertsClient, SendResult};
pub use event::Event;

use std::sync::OnceLock;

static GLOBAL_CLIENT: OnceLock<ApiAlertsClient> = OnceLock::new();

/// Initialise the global client with the given API key.
///
/// Subsequent calls are silently ignored — the first call wins.
///
/// # Example
///
/// ```rust,no_run
/// apialerts::configure("your-api-key");
/// ```
pub fn configure(api_key: impl Into<String>) {
    let _ = GLOBAL_CLIENT.set(ApiAlertsClient::new(api_key));
}

/// Initialise the global client with first-party overrides (integration name,
/// version, and base URL).
///
/// Used internally by official integrations and in tests to redirect requests
/// to a mock server.
pub fn configure_with_overrides(
    api_key: impl Into<String>,
    integration: impl Into<String>,
    version: impl Into<String>,
    base_url: impl Into<String>,
) {
    let client = ApiAlertsClient::new(api_key).set_overrides(integration, version, base_url);
    let _ = GLOBAL_CLIENT.set(client);
}

/// Send an event — fire-and-forget. Never panics.
///
/// Critical errors (not configured, missing key, empty message) are always
/// logged to stderr. HTTP errors and successes are only logged when debug is
/// enabled.
///
/// # Example
///
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() {
/// apialerts::configure("your-api-key");
/// apialerts::send(apialerts::Event::new("Deploy complete")).await;
/// # }
/// ```
pub async fn send(event: Event) {
    match GLOBAL_CLIENT.get() {
        Some(client) => client.send(event).await,
        None => eprintln!("x (apialerts.com) Error: client not configured"),
    }
}

/// Send an event and return the result. Never returns an error — check
/// `result.success` instead.
///
/// # Example
///
/// ```rust,no_run
/// # #[tokio::main]
/// # async fn main() {
/// apialerts::configure("your-api-key");
/// let result = apialerts::send_async(apialerts::Event::new("Deploy complete")).await;
/// if result.success {
///     println!("Sent to {} ({})", result.workspace.unwrap(), result.channel.unwrap());
/// } else {
///     eprintln!("Error: {}", result.error.unwrap());
/// }
/// # }
/// ```
pub async fn send_async(event: Event) -> SendResult {
    match GLOBAL_CLIENT.get() {
        Some(client) => client.send_async(event).await,
        None => SendResult::from_error("client not configured"),
    }
}
