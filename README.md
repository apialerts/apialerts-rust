# API Alerts • Rust Client

[![Crates.io](https://img.shields.io/crates/v/apialerts)](https://crates.io/crates/apialerts)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[Crates.io](https://crates.io/crates/apialerts) • [GitHub](https://github.com/apialerts/apialerts-rust) • [API Alerts](https://apialerts.com)

Effortless project notifications. Send once, deliver everywhere.

## Installation

```toml
[dependencies]
apialerts = "1.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

## Quick Start

```rust
use apialerts::{ApiAlertsClient, Event};

#[tokio::main]
async fn main() {
    let client = ApiAlertsClient::new("your-api-key");
    client.send(Event::new("Deploy complete")).await;
}
```

## Usage

The Rust SDK is instance-based - construct `ApiAlertsClient` directly and manage
its lifetime yourself. This is more idiomatic for Rust than a global singleton.

```rust
use apialerts::{ApiAlertsClient, Event};

#[tokio::main]
async fn main() {
    let client = ApiAlertsClient::new("your-api-key");

    // Fire-and-forget - never panics
    client.send(Event::new("Deploy complete")).await;

    // Or get the result back
    match client.send_async(Event::new("Deploy complete")).await {
        Ok(result) => {
            println!("Sent to {} ({})", result.workspace, result.channel);
            for w in &result.warnings { println!("Warning: {}", w); }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Debug Logging

```rust
let mut client = ApiAlertsClient::new("your-api-key");
client.set_debug(true); // logs successful sends and errors to stderr
```

### Overrides

Use `set_overrides` to change the integration name, version, or base URL
(useful for official integrations and testing).

```rust
let mut client = ApiAlertsClient::new("your-api-key");
client.set_overrides("my-integration", "1.0.0", "https://api.apialerts.com");
```

### Event Fields

Only `message` is required. All other fields are optional.

| Field        | Type                 | Required | Description                      |
|--------------|----------------------|----------|----------------------------------|
| `new(s)`     | `&str`               | Yes      | Main notification message        |
| `.channel()` | `&str`               | No       | Target channel name              |
| `.event()`   | `&str`               | No       | Event key for routing            |
| `.title()`   | `&str`               | No       | Short title                      |
| `.tags()`    | `Vec<&str>`          | No       | Categorisation tags              |
| `.link()`    | `&str`               | No       | URL associated with the event (deeplink + CTA) |
| `.data()`    | `serde_json::Value`  | No       | Arbitrary key-value metadata     |

```rust
use apialerts::Event;

let event = Event::new("Deploy complete")
    .channel("releases")
    .event("ci.deploy")
    .title("Deployed")
    .tags(vec!["CI/CD", "Rust"])
    .link("https://github.com/apialerts/apialerts-rust/actions")
    .data(serde_json::json!({ "version": "2.0.0" }));
```

## Links

- [Documentation](https://apialerts.com/docs)
- [Sign up](https://apialerts.com)
- [GitHub Issues](https://github.com/apialerts/apialerts-rust/issues)
