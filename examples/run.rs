use apialerts::{configure, send_async, Event};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let _flag = args.get(1).map(|s| s.as_str());

    let api_key = std::env::var("APIALERTS_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        eprintln!("Error: APIALERTS_API_KEY environment variable is not set");
        std::process::exit(1);
    }
    configure(api_key);

    // Minimal send — message only
    let result = send_async(Event::new("Rust SDK - minimal")).await;
    if result.success {
        println!(
            "✓ sent to {} ({})",
            result.workspace.as_deref().unwrap_or(""),
            result.channel.as_deref().unwrap_or("")
        );
    } else {
        eprintln!("Error (minimal): {}", result.error.as_deref().unwrap_or("unknown"));
        std::process::exit(1);
    }

    // Full send — all fields
    let result = send_async(
        Event::new("Rust SDK - full")
            .channel("developer")
            .event("sdk.test")
            .title("Integration Test")
            .tags(vec!["CI/CD", "Rust"])
            .link("https://github.com/apialerts/apialerts-rust/actions")
            .data(serde_json::json!({ "version": "2.0.0" })),
    )
    .await;

    if result.success {
        println!(
            "✓ sent to {} ({})",
            result.workspace.as_deref().unwrap_or(""),
            result.channel.as_deref().unwrap_or("")
        );
        for w in &result.warnings {
            println!("! warning: {}", w);
        }
    } else {
        eprintln!("Error (full): {}", result.error.as_deref().unwrap_or("unknown"));
        std::process::exit(1);
    }
}
