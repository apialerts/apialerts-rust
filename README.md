# API Alerts • Rust Client

[![Crates.io](https://img.shields.io/crates/v/apialerts)](https://crates.io/crates/apialerts)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[Crates.io](https://crates.io/crates/apialerts) • [GitHub](https://github.com/apialerts/apialerts-rust) • [API Alerts](https://apialerts.com)

Effortless project notifications. Send once, deliver everywhere.

## Installation

```toml
[dependencies]
apialerts = "2.0.0"
```

## Quick Start

```rust
use apialerts::Event;

#[tokio::main]
async fn main() {
    apialerts::configure("your-api-key");

    apialerts::send(Event::new("Deploy complete")).await;
}
```

## Usage

### Global singleton (recommended)

Call `configure` once at startup, then use the top-level `send` / `send_async`
functions anywhere in your app.

```rust
use apialerts::Event;

#[tokio::main]
async fn main() {
    apialerts::configure("your-api-key");

    // Fire-and-forget — never panics
    apialerts::send(Event::new("Deploy complete")).await;

    // Or get the result back — never returns an error, check result.success
    let result = apialerts::send_async(Event::new("Deploy complete")).await;
    if result.success {
        println!("Sent to {} ({})", result.workspace.unwrap(), result.channel.unwrap());
    } else {
        eprintln!("Error: {}", result.error.unwrap());
    }
}
```

### Event Fields

Only `message` is required. All other fields are optional.

| Field       | Type                 | Required | Description                      |
|-------------|----------------------|----------|----------------------------------|
| `new(s)`    | `&str`               | Yes      | Main notification message        |
| `.channel()`| `&str`               | No       | Target channel name              |
| `.event()`  | `&str`               | No       | Event key for routing            |
| `.title()`  | `&str`               | No       | Short title                      |
| `.tags()`   | `Vec<&str>`          | No       | Categorisation tags              |
| `.link()`   | `&str`               | No       | URL attached to the notification |
| `.data()`   | `serde_json::Value`  | No       | Arbitrary key-value metadata     |

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

### Instance-based client

Use `ApiAlertsClient` directly when you need multiple clients or want to manage
the lifecycle yourself.

```rust
use apialerts::{ApiAlertsClient, Event};

#[tokio::main]
async fn main() {
    let client = ApiAlertsClient::new("your-api-key").debug(true);

    let result = client.send_async(Event::new("Deploy complete")).await;
    if result.success {
        println!("Sent to {} ({})", result.workspace.unwrap(), result.channel.unwrap());
    }
}
```

## Links

- [Documentation](https://apialerts.com/docs)
- [Sign up](https://apialerts.com)
- [GitHub Issues](https://github.com/apialerts/apialerts-rust/issues)
