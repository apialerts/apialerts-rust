### API Alerts - Rust

This is a simple API for sending alerts to the [API Alerts](https://apialerts.com) service.

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apialerts = "2.0.0"
```

### Usage

```rust
use apialerts::{event::ApiAlertsEvent, ApiAlertsClient};

#[tokio::main]
async fn main() {
    let client = ApiAlertsClient::new("{API-KEY}".to_string());

    let event = ApiAlertsEvent::new(
        "rust channel".to_string(),
        "I've caught the rust bug".to_string(),
    );

    // Blocking - using futures
    match client.send(event) {
        Ok(_) => {}
        Err(e) => println!("Error sending alert: {:?}", e),
    }

    // OR

    // Async - using async-std
    match client.send_async(event).await {
        Ok(_) => {}
        Err(e) => println!("Error sending alert: {:?}", e),
    }
}
```

You can also use the `update_config` and `update_api_key` methods to update the configuration of the client.

```rust
use apialerts::{event::ApiAlertsEvent, ApiAlertsClient};

#[tokio::main]
async fn main() {
    let client = ApiAlertsClient::new("{API-KEY}".to_string())
        .update_config(ApiAlertsConfig::new_default_config())
        .update_api_key("{API-KEY}".to_string());

    //...
}
```

You can also set the tags and links on the alert event.

```rust
use apialerts::{event::ApiAlertsEvent, ApiAlertsClient};

#[tokio::main]
async fn main() {
    //...

    let event = ApiAlertsEvent::new(
        "rust channel".to_string(),
        "I've caught the rust bug".to_string(),
    )
    .set_tags(vec!["rust", "bug".to_string()])
    .set_links(vec!["https://github.com/apialerts/apialerts-rust".to_string()]);

    //...
}
```
