use serde::Serialize;

/// An event to send to the API Alerts platform.
///
/// Only `message` is required. All other fields are optional and will be
/// omitted from the JSON payload if not set — equivalent to Go's `omitempty`.
///
/// # Example
///
/// ```rust
/// use apialerts::Event;
///
/// // Minimal
/// let event = Event::new("Deploy complete");
///
/// // Full
/// let event = Event::new("Deploy complete")
///     .channel("releases")
///     .event("ci.deploy")
///     .title("Deployed")
///     .tags(vec!["CI/CD", "Rust"])
///     .link("https://github.com/apialerts/apialerts-rust/actions")
///     .data(serde_json::json!({ "version": "2.0.0" }));
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,

    // Field is named event_key internally to avoid conflict with the builder
    // method name. Serializes as "event" in JSON.
    #[serde(rename = "event", skip_serializing_if = "Option::is_none")]
    pub event_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Event {
    /// Create a new event with only a message. All other fields default to `None`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            channel: None,
            event_key: None,
            title: None,
            tags: None,
            link: None,
            data: None,
        }
    }

    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn event(mut self, event: impl Into<String>) -> Self {
        self.event_key = Some(event.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = Some(tags.into_iter().map(|t| t.into()).collect());
        self
    }

    pub fn link(mut self, link: impl Into<String>) -> Self {
        self.link = Some(link.into());
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}
