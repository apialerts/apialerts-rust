use serde_json::{json, Value};

pub struct ApiAlertsEvent {
    pub channel: String,
    pub message: String,
    pub tags: Option<Vec<String>>,
    pub link: Option<String>,
}

impl ApiAlertsEvent {
    pub fn new(channel: String, message: String) -> Self {
        ApiAlertsEvent {
            channel,
            message,
            tags: None,
            link: None,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_link(mut self, link: String) -> Self {
        self.link = Some(link);
        self
    }

    pub fn convert_to_json(self) -> Value {
        json!({
            "channel": self.channel,
            "message": self.message,
            "link": self.link,
            "tags": self.tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn test_base_event_build() {
        let event = ApiAlertsEvent::new("channel".to_string(), "message".to_string());
        assert_eq!(event.channel, "channel");
        assert_eq!(event.message, "message");
        assert!(event.tags.is_none());
        assert!(event.link.is_none());
    }

    #[test]
    fn test_event_with_tags_and_link() {
        let event = ApiAlertsEvent::new("channel".to_string(), "message".to_string())
            .with_link("http://waffles.yeet".to_string())
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(event.channel, "channel");
        assert_eq!(event.message, "message");
        assert_eq!(
            event.tags.unwrap(),
            vec!["tag1".to_string(), "tag2".to_string()]
        );
        assert_eq!(event.link.unwrap(), "http://waffles.yeet");
    }

    #[test]
    fn test_convert_to_json() {
        let event = ApiAlertsEvent::new("channel".to_string(), "message".to_string())
            .with_link("http://waffles.yeet".to_string())
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);
        let json = event.convert_to_json();
        assert_eq!(json["channel"], "channel");
        assert_eq!(json["message"], "message");
        assert_eq!(json["link"], "http://waffles.yeet");

        if let Some(tags) = json["tags"].as_array() {
            let tags_vec: Vec<String> = tags
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            assert_eq!(tags_vec, vec!["tag1".to_string(), "tag2".to_string()])
        } else {
            panic!("Tags should be an array");
        }
    }
}
