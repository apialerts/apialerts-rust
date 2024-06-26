use serde_json::{json, Value};
use std::env;
use std::error::Error;
use ureq::{Agent, AgentBuilder};

struct Client {
    api_key: String,
    agent: Agent,
}

impl Client {
    fn new() -> Self {
        Client {
            api_key: env::var("APIALERTS_API_KEY").unwrap_or_default(),
            agent: AgentBuilder::new().build(),
        }
    }

    fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    fn send(&self, message: &str, tags: &[String], link: &str) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("api key is missing".into());
        }
        if message.is_empty() {
            return Err("message is required".into());
        }

        let payload = json!({
            "message": message,
            "tags": tags,
            "link": link,
        });

        let url = "https://api.apialerts.com/event";

        let response = self
            .agent
            .post(url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .set("X-Integration", "rust")
            .set("X-Version", "1.0.0")
            .send_string(&payload.to_string())?;

        match response.status() {
            200 => {
                let data: Value = serde_json::from_reader(response.into_reader())?;
                println!(
                    "✓ (apialerts.com) Alert sent to {} successfully.",
                    data["project"]
                );
                Ok(())
            }
            400 => Err("bad request".into()),
            401 => Err("unauthorized".into()),
            403 => Err("forbidden".into()),
            429 => Err("rate limit exceeded".into()),
            _ => Err("unknown error".into()),
        }
    }
}

fn api_alerts_client() -> Client {
    Client::new()
}

use std::thread;

fn main() {
    let mut client = api_alerts_client();
    client.set_api_key("89fee423-7fd4-4dcb-a234-23c87fb11a8e".to_string());

    let handle = thread::spawn(move || {
        let result = client.send(
            "Rust Test Message",
            &["Rust is awesome".to_string()],
            "https://github.com/apialerts/",
        );

        if let Err(e) = result {
            println!("Error: {}", e);
        }
    });

    handle.join().unwrap();
}
