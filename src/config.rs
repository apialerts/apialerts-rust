pub struct ApiAlertsConfig {
    pub logging: bool,
    pub timeout: u8,
    pub debug: bool,
}

impl ApiAlertsConfig {
    pub fn new_default_config() -> Self {
        ApiAlertsConfig {
            logging: true,
            timeout: 30,
            debug: false,
        }
    }
}
