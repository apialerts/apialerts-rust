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
    let mut client = ApiAlertsClient::new("test-api-key");
    client.set_overrides("rust-test", "2.0.0", format!("{}/event", server.uri()));
    client
}

// ── Validation ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_missing_message_returns_error() {
    let server = start_server().await;
    let client = client_for(&server);
    let result = client.send_async(Event::new("")).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "message is required");
}

#[tokio::test]
async fn test_missing_api_key_returns_error() {
    let server = start_server().await;
    let mut client = ApiAlertsClient::new("");
    client.set_overrides("rust-test", "2.0.0", format!("{}/event", server.uri()));
    let result = client.send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "api key is missing");
}

// ── HTTP status codes ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_200_returns_send_result() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(success_body("My Workspace", "general")),
        )
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;

    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.workspace, "My Workspace");
    assert_eq!(r.channel, "general");
    assert!(r.warnings.is_empty());
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

    let result = client_for(&server).send_async(Event::new("test")).await;

    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.warnings.len(), 1);
    assert_eq!(r.warnings[0], "This channel will be deprecated soon");
}

#[tokio::test]
async fn test_400_returns_bad_request() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "bad request");
}

#[tokio::test]
async fn test_401_returns_unauthorized() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "unauthorized - check your api key"
    );
}

#[tokio::test]
async fn test_403_returns_forbidden() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "forbidden");
}

#[tokio::test]
async fn test_429_returns_rate_limit_exceeded() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "rate limit exceeded");
}

#[tokio::test]
async fn test_500_returns_unexpected_status() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "unexpected status: 500");
}

#[tokio::test]
async fn test_invalid_json_response_returns_invalid_response() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid response from server"
    );
}

// ── Request headers ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_authorization_header_is_sent() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("W", "C")))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_integration_headers_are_sent() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("X-Integration", "rust-test"))
        .and(header("X-Version", "2.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("W", "C")))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("test")).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_overrides_changes_headers() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("X-Integration", "github-actions"))
        .and(header("X-Version", "1.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("W", "C")))
        .mount(&server)
        .await;

    let mut client = ApiAlertsClient::new("test-api-key");
    client.set_overrides("github-actions", "1.0.0", format!("{}/event", server.uri()));
    let result = client.send_async(Event::new("test")).await;

    assert!(result.is_ok());
}

// ── Payload serialization ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_full_event_payload() {
    let server = start_server().await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header_exists("Authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("W", "developer")))
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

    assert!(result.is_ok());
    assert_eq!(result.unwrap().channel, "developer");
}

#[tokio::test]
async fn test_null_fields_are_omitted_from_payload() {
    let server = start_server().await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("W", "C")))
        .mount(&server)
        .await;

    let result = client_for(&server).send_async(Event::new("minimal")).await;
    assert!(result.is_ok());

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
async fn test_send_async_with_key_uses_provided_key() {
    let server = start_server().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .and(header("Authorization", "Bearer override-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success_body("W", "C")))
        .mount(&server)
        .await;

    let mut client = ApiAlertsClient::new("original-key");
    client.set_overrides("rust-test", "2.0.0", format!("{}/event", server.uri()));
    let result = client
        .send_async_with_key("override-key", Event::new("test"))
        .await;

    assert!(result.is_ok());
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

    // send() swallows errors - should complete without panicking
    client_for(&server).send(Event::new("test")).await;
}
