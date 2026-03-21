use apialerts::{configure, send_async, Event};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let is_build             = args.contains(&"--build".to_string());
    let is_release           = args.contains(&"--release".to_string());
    let is_publish           = args.contains(&"--publish".to_string());
    let is_integration_tests = args.contains(&"--integration-tests".to_string());

    let channel_idx = args.iter().position(|a| a == "--channel");
    let channel = channel_idx
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("testing");

    let api_key = std::env::var("APIALERTS_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        eprintln!("Error: APIALERTS_API_KEY environment variable is not set");
        std::process::exit(1);
    }
    configure(api_key);

    let link = "https://github.com/apialerts/apialerts-rust/actions";

    if is_build {
        let result = send_async(
            Event::new("Rust SDK - PR build success")
                .channel("developer")
                .event("ci.build")
                .title("Build Passed")
                .tags(vec!["CI/CD", "Rust", "Build"])
                .link(link),
        )
        .await;
        if result.success {
            println!("✓ Sent to {} ({})", result.workspace.as_deref().unwrap_or(""), result.channel.as_deref().unwrap_or(""));
        } else {
            eprintln!("Error: {}", result.error.as_deref().unwrap_or("unknown"));
            std::process::exit(1);
        }

    } else if is_release {
        let result = send_async(
            Event::new("Rust SDK - Build for publish success")
                .channel("developer")
                .event("ci.release")
                .title("Release Build Passed")
                .tags(vec!["CI/CD", "Rust", "Build"])
                .link(link),
        )
        .await;
        if result.success {
            println!("✓ Sent to {} ({})", result.workspace.as_deref().unwrap_or(""), result.channel.as_deref().unwrap_or(""));
        } else {
            eprintln!("Error: {}", result.error.as_deref().unwrap_or("unknown"));
            std::process::exit(1);
        }

    } else if is_publish {
        let result = send_async(
            Event::new("Rust SDK - crates.io publish success")
                .channel("releases")
                .event("ci.publish")
                .title("Published")
                .tags(vec!["CI/CD", "Rust", "Deploy"])
                .link(link),
        )
        .await;
        if result.success {
            println!("✓ Sent to {} ({})", result.workspace.as_deref().unwrap_or(""), result.channel.as_deref().unwrap_or(""));
        } else {
            eprintln!("Error: {}", result.error.as_deref().unwrap_or("unknown"));
            std::process::exit(1);
        }

    } else if is_integration_tests {
        let result = send_async(Event::new("Rust SDK - minimal").channel(channel)).await;
        if result.success {
            println!("✓ sent to {} ({})", result.workspace.as_deref().unwrap_or(""), result.channel.as_deref().unwrap_or(""));
        } else {
            eprintln!("Error (minimal): {}", result.error.as_deref().unwrap_or("unknown"));
            std::process::exit(1);
        }

        let result = send_async(
            Event::new("Rust SDK - full")
                .channel(channel)
                .event("sdk.test")
                .title("Integration Test")
                .tags(vec!["CI/CD", "Rust"])
                .link(link)
                .data(serde_json::json!({ "version": "2.0.0" })),
        )
        .await;
        if result.success {
            println!("✓ sent to {} ({})", result.workspace.as_deref().unwrap_or(""), result.channel.as_deref().unwrap_or(""));
            for w in &result.warnings {
                println!("! Warning: {}", w);
            }
        } else {
            eprintln!("Error (full): {}", result.error.as_deref().unwrap_or("unknown"));
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: pass --build, --release, --publish, or --integration-tests");
        std::process::exit(1);
    }
}
