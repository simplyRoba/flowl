use std::env;

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
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: parse_env("FLOWL_PORT", 4100),
            db_path: env::var("FLOWL_DB_PATH").unwrap_or_else(|_| "/data/flowl.db".to_string()),
            mqtt_host: env::var("FLOWL_MQTT_HOST").unwrap_or_else(|_| "localhost".to_string()),
            mqtt_port: parse_env("FLOWL_MQTT_PORT", 1883),
            mqtt_topic_prefix: env::var("FLOWL_MQTT_TOPIC_PREFIX")
                .unwrap_or_else(|_| "flowl".to_string()),
            log_level: env::var("FLOWL_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            mqtt_disabled: parse_env("FLOWL_MQTT_DISABLED", false),
            ai_api_key: env::var("FLOWL_AI_API_KEY").ok(),
            ai_base_url: env::var("FLOWL_AI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            ai_model: env::var("FLOWL_AI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
        }
    }
}

fn parse_env<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    unsafe fn clear_flowl_env() {
        for key in [
            "FLOWL_PORT",
            "FLOWL_DB_PATH",
            "FLOWL_MQTT_HOST",
            "FLOWL_MQTT_PORT",
            "FLOWL_MQTT_TOPIC_PREFIX",
            "FLOWL_LOG_LEVEL",
            "FLOWL_MQTT_DISABLED",
            "FLOWL_AI_API_KEY",
            "FLOWL_AI_BASE_URL",
            "FLOWL_AI_MODEL",
        ] {
            unsafe { env::remove_var(key) };
        }
    }

    #[test]
    fn defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { clear_flowl_env() };

        let config = Config::from_env();
        assert_eq!(config.port, 4100);
        assert_eq!(config.db_path, "/data/flowl.db");
        assert_eq!(config.mqtt_host, "localhost");
        assert_eq!(config.mqtt_port, 1883);
        assert_eq!(config.mqtt_topic_prefix, "flowl");
        assert_eq!(config.log_level, "info");
        assert!(config.ai_api_key.is_none());
        assert_eq!(config.ai_base_url, "https://api.openai.com/v1");
        assert_eq!(config.ai_model, "gpt-4o-mini");
    }

    #[test]
    fn custom_values() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("FLOWL_PORT", "3000");
            env::set_var("FLOWL_DB_PATH", "/tmp/test.db");
            env::set_var("FLOWL_MQTT_HOST", "broker.local");
            env::set_var("FLOWL_MQTT_PORT", "1884");
            env::set_var("FLOWL_MQTT_TOPIC_PREFIX", "myplants");
            env::set_var("FLOWL_LOG_LEVEL", "debug");
            env::set_var("FLOWL_MQTT_DISABLED", "true");
            env::set_var("FLOWL_AI_API_KEY", "sk-test-key");
            env::set_var("FLOWL_AI_BASE_URL", "http://localhost:11434/v1");
            env::set_var("FLOWL_AI_MODEL", "llama3");
        }

        let config = Config::from_env();
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

        unsafe { clear_flowl_env() };
    }

    #[test]
    fn mqtt_disabled_defaults_false() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { clear_flowl_env() };

        let config = Config::from_env();
        assert!(!config.mqtt_disabled);
    }

    #[test]
    fn invalid_mqtt_disabled_falls_back_to_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { clear_flowl_env() };
        unsafe { env::set_var("FLOWL_MQTT_DISABLED", "not_a_bool") };

        let config = Config::from_env();
        assert!(!config.mqtt_disabled);

        unsafe { clear_flowl_env() };
    }

    #[test]
    fn ai_api_key_absent_results_in_none() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { clear_flowl_env() };

        let config = Config::from_env();
        assert!(config.ai_api_key.is_none());
    }

    #[test]
    fn invalid_port_falls_back_to_default() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { clear_flowl_env() };
        unsafe { env::set_var("FLOWL_PORT", "not_a_number") };
        let config = Config::from_env();
        assert_eq!(config.port, 4100);
        unsafe { env::remove_var("FLOWL_PORT") };
    }
}
