use apialerts::{ApiAlertsClient, Event};
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn success_body(workspace: &str, channel: &str) -> serde_json::Value {
    serde_json::json!({
        "workspace": workspace,
        "channel": channel,
        "warnings": []
    })
}

fn success_body_with_warnings(workspace: &str, channel: &str) -> serde_json::Value {
    serde_json::json!({
        "workspace": workspace,
        "channel": channel,
        "warnings": ["This channel will be deprecated soon"]
    })
}

async fn start_server() -> MockServer {
    MockServer::start().await
}

fn client_for(server: &MockServer) -> ApiAlertsClient {
    ApiAlertsClient::new("test-api-key")
        .set_overrides("rust-test", "2.0.0", format!("{}/event", server.uri()))
}

// ── Validation ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_missing_message_returns_error() {
    let server = start_server().await;
    let client = client_for(&server);
    let result = client.send_async(Event::new("")).await;
    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("message is required"));
}

#[tokio::test]
async fn test_missing_api_key_returns_error() {
    let server = start_server().await;
    let client = ApiAlertsClient::new("")
        .set_overrides("rust-test", "2.0.0", format!("{}/event", server.uri()));
    let result = client.send_async(Event::new("test")).await;
    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("api key is missing"));
}

// ── HTTP status codes ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_200_returns_send_result() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("My Workspace", "general")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(result.success);
    assert_eq!(result.workspace.as_deref(), Some("My Workspace"));
    assert_eq!(result.channel.as_deref(), Some("general"));
    assert!(result.warnings.is_empty());
}

#[tokio::test]
async fn test_200_with_warnings() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body_with_warnings("My Workspace", "general")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(result.success);
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0], "This channel will be deprecated soon");
}

#[tokio::test]
async fn test_400_returns_bad_request() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("bad request"));
}

#[tokio::test]
async fn test_401_returns_unauthorized() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("unauthorized — check your api key"));
}

#[tokio::test]
async fn test_403_returns_forbidden() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("forbidden"));
}

#[tokio::test]
async fn test_429_returns_rate_limit_exceeded() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("rate limit exceeded"));
}

#[tokio::test]
async fn test_500_returns_unexpected_status() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("unexpected status: 500"));
}

#[tokio::test]
async fn test_invalid_json_response_returns_invalid_response() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("invalid response from server"));
}

// ── Request headers ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_authorization_header_is_sent() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("W", "C")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(result.success);
}

#[tokio::test]
async fn test_integration_headers_are_sent() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("X-Integration", "rust-test"))
        .and(header("X-Version", "2.0.0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("W", "C")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("test"))
        .await;

    assert!(result.success);
}

#[tokio::test]
async fn test_set_overrides_changes_headers() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("X-Integration", "github-actions"))
        .and(header("X-Version", "1.0.0"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("W", "C")),
        )
        .mount(&server)
        .await;

    let result = ApiAlertsClient::new("test-api-key")
        .set_overrides("github-actions", "1.0.0", format!("{}/event", server.uri()))
        .send_async(Event::new("test"))
        .await;

    assert!(result.success);
}

// ── Payload serialization ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_full_event_payload() {
    let server = start_server().await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header_exists("Authorization"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("W", "developer")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(
            Event::new("Full payload test")
                .channel("developer")
                .event("ci.deploy")
                .title("Deployed")
                .tags(vec!["CI/CD", "Rust"])
                .link("https://github.com")
                .data(serde_json::json!({ "version": "2.0.0" })),
        )
        .await;

    assert!(result.success);
    assert_eq!(result.channel.as_deref(), Some("developer"));
}

#[tokio::test]
async fn test_null_fields_are_omitted_from_payload() {
    let server = start_server().await;

    // wiremock records requests; verify the body does not contain null fields
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("W", "C")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server)
        .send_async(Event::new("minimal"))
        .await;

    assert!(result.success);

    let received = &server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&received.body).unwrap();
    assert!(body.get("channel").is_none());
    assert!(body.get("event").is_none());
    assert!(body.get("title").is_none());
    assert!(body.get("tags").is_none());
    assert!(body.get("link").is_none());
    assert!(body.get("data").is_none());
}

#[tokio::test]
async fn test_send_with_key_uses_provided_key() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("Authorization", "Bearer override-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(success_body("W", "C")),
        )
        .mount(&server)
        .await;

    let result = ApiAlertsClient::new("original-key")
        .set_overrides("rust-test", "2.0.0", format!("{}/event", server.uri()))
        .send_with_key("override-key", Event::new("test"))
        .await;

    assert!(result.success);
}

// ── Fire-and-forget ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_does_not_panic_on_error() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    // send() swallows errors — should complete without panicking
    client_for(&server).send(Event::new("test")).await;
}
