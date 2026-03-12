use std::env;

pub trait ConfigSource {
    fn get(&self, key: &str) -> Option<String>;
}

struct EnvConfigSource;

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub db_path: String,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub mqtt_topic_prefix: String,
    pub log_level: String,
    pub mqtt_disabled: bool,
    pub ai_api_key: Option<String>,
    pub ai_base_url: String,
    pub ai_model: String,
    pub ai_rate_limit: u32,
}

impl Config {
    pub fn load() -> Self {
        Self::load_from(&EnvConfigSource)
    }

    pub fn load_from(source: &impl ConfigSource) -> Self {
        Self {
            port: parse_or(source, "FLOWL_PORT", 4100),
            db_path: source
                .get("FLOWL_DB_PATH")
                .unwrap_or_else(|| "/data/flowl.db".to_string()),
            mqtt_host: source
                .get("FLOWL_MQTT_HOST")
                .unwrap_or_else(|| "localhost".to_string()),
            mqtt_port: parse_or(source, "FLOWL_MQTT_PORT", 1883),
            mqtt_topic_prefix: source
                .get("FLOWL_MQTT_TOPIC_PREFIX")
                .unwrap_or_else(|| "flowl".to_string()),
            log_level: source
                .get("FLOWL_LOG_LEVEL")
                .unwrap_or_else(|| "info".to_string()),
            mqtt_disabled: parse_or(source, "FLOWL_MQTT_DISABLED", false),
            ai_api_key: source.get("FLOWL_AI_API_KEY"),
            ai_base_url: source
                .get("FLOWL_AI_BASE_URL")
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            ai_model: source
                .get("FLOWL_AI_MODEL")
                .unwrap_or_else(|| "gpt-4.1-mini".to_string()),
            ai_rate_limit: parse_or(source, "FLOWL_AI_RATE_LIMIT", 10),
        }
    }
}

fn parse_or<T: std::str::FromStr>(source: &impl ConfigSource, key: &str, default: T) -> T {
    source
        .get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockConfig(HashMap<&'static str, &'static str>);

    impl MockConfig {
        fn new() -> Self {
            Self(HashMap::new())
        }

        fn with(mut self, key: &'static str, value: &'static str) -> Self {
            self.0.insert(key, value);
            self
        }
    }

    impl ConfigSource for MockConfig {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).map(|s| (*s).to_string())
        }
    }

    #[test]
    fn defaults() {
        let config = Config::load_from(&MockConfig::new());
        assert_eq!(config.port, 4100);
        assert_eq!(config.db_path, "/data/flowl.db");
        assert_eq!(config.mqtt_host, "localhost");
        assert_eq!(config.mqtt_port, 1883);
        assert_eq!(config.mqtt_topic_prefix, "flowl");
        assert_eq!(config.log_level, "info");
        assert!(!config.mqtt_disabled);
        assert!(config.ai_api_key.is_none());
        assert_eq!(config.ai_base_url, "https://api.openai.com/v1");
        assert_eq!(config.ai_model, "gpt-4.1-mini");
        assert_eq!(config.ai_rate_limit, 10);
    }

    #[test]
    fn custom_values() {
        let config = Config::load_from(
            &MockConfig::new()
                .with("FLOWL_PORT", "3000")
                .with("FLOWL_DB_PATH", "/tmp/test.db")
                .with("FLOWL_MQTT_HOST", "broker.local")
                .with("FLOWL_MQTT_PORT", "1884")
                .with("FLOWL_MQTT_TOPIC_PREFIX", "myplants")
                .with("FLOWL_LOG_LEVEL", "debug")
                .with("FLOWL_MQTT_DISABLED", "true")
                .with("FLOWL_AI_API_KEY", "sk-test-key")
                .with("FLOWL_AI_BASE_URL", "http://localhost:11434/v1")
                .with("FLOWL_AI_MODEL", "llama3")
                .with("FLOWL_AI_RATE_LIMIT", "20"),
        );
        assert_eq!(config.port, 3000);
        assert_eq!(config.db_path, "/tmp/test.db");
        assert_eq!(config.mqtt_host, "broker.local");
        assert_eq!(config.mqtt_port, 1884);
        assert_eq!(config.mqtt_topic_prefix, "myplants");
        assert_eq!(config.log_level, "debug");
        assert!(config.mqtt_disabled);
        assert_eq!(config.ai_api_key.as_deref(), Some("sk-test-key"));
        assert_eq!(config.ai_base_url, "http://localhost:11434/v1");
        assert_eq!(config.ai_model, "llama3");
        assert_eq!(config.ai_rate_limit, 20);
    }

    #[test]
    fn invalid_mqtt_disabled_falls_back_to_default() {
        let config =
            Config::load_from(&MockConfig::new().with("FLOWL_MQTT_DISABLED", "not_a_bool"));
        assert!(!config.mqtt_disabled);
    }

    #[test]
    fn ai_rate_limit_zero_disables() {
        let config = Config::load_from(&MockConfig::new().with("FLOWL_AI_RATE_LIMIT", "0"));
        assert_eq!(config.ai_rate_limit, 0);
    }

    #[test]
    fn invalid_port_falls_back_to_default() {
        let config = Config::load_from(&MockConfig::new().with("FLOWL_PORT", "not_a_number"));
        assert_eq!(config.port, 4100);
    }
}
