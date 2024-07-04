### API Alerts - Rust

This is a simple API for sending alerts to the [API Alerts](https://api-alerts.com) service.

### Usage

```rust
extern crate api_alerts;

use apialerts::api_alerts_client;

fn main() {
    let mut client = api_alerts_client();

    // Set API key (you might want to use an environment variable in practice)
    client.set_api_key("{{API_KEY}}".to_string());

    // Test sending an alert
    let message = "Test alert from Rust library";
    let tags = vec!["test".to_string(), "rust".to_string()];
    let link = "https://example.com";

    match client.send(message, &tags, link) {
        Ok(()) => println!("Alert sent successfully"),
        Err(e) => eprintln!("Failed to send alert: {}", e),
    }
}
```
