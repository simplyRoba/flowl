use std::env;

pub use crate::auth::config::{AuthConfigError, OidcConfig as EnabledAuthConfig};

pub trait ConfigSource {
    fn get(&self, key: &str) -> Option<String>;

    /// Reads an authentication setting without discarding invalid Unicode.
    ///
    /// # Errors
    ///
    /// Returns `VarError::NotUnicode` for a present non-UTF-8 value.
    fn get_auth(&self, key: &str) -> Result<Option<String>, env::VarError> {
        Ok(self.get(key))
    }
}

struct EnvConfigSource;

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    fn get_auth(&self, key: &str) -> Result<Option<String>, env::VarError> {
        match env::var(key) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(error @ env::VarError::NotUnicode(_)) => Err(error),
        }
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

/// Strict, authentication-specific configuration. Ordinary [`Config`] values remain lenient.
#[derive(Clone)]
pub enum AuthConfig {
    Disabled,
    Enabled(Box<EnabledAuthConfig>),
}

impl AuthConfig {
    /// # Errors
    ///
    /// Returns an error when `FLOWL_AUTH_ENABLED` is malformed or enabled OIDC configuration is
    /// incomplete or unsafe.
    pub fn load() -> Result<Self, AuthConfigError> {
        Self::load_from(&EnvConfigSource)
    }

    /// # Errors
    ///
    /// Returns an error when `FLOWL_AUTH_ENABLED` is malformed or enabled OIDC configuration is
    /// incomplete or unsafe.
    pub fn load_from(source: &impl ConfigSource) -> Result<Self, AuthConfigError> {
        crate::auth::config::load_from(&crate::auth::app::AuthSource(source))
            .map(|config| config.map_or(Self::Disabled, |config| Self::Enabled(Box::new(config))))
    }

    pub fn enabled(&self) -> Option<&EnabledAuthConfig> {
        match self {
            Self::Disabled => None,
            Self::Enabled(config) => Some(config.as_ref()),
        }
    }
}

fn parse_or<T: std::str::FromStr>(source: &impl ConfigSource, key: &str, default: T) -> T {
    source
        .get(key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::http::HeaderMap;

    use super::*;

    struct MockConfig(HashMap<String, String>);

    impl MockConfig {
        fn new() -> Self {
            Self(HashMap::new())
        }

        fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.0.insert(key.into(), value.into());
            self
        }
    }

    impl ConfigSource for MockConfig {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
    }

    struct NonUnicodeConfig {
        inner: MockConfig,
        invalid_key: &'static str,
    }

    impl ConfigSource for NonUnicodeConfig {
        fn get(&self, key: &str) -> Option<String> {
            if key == self.invalid_key {
                None
            } else {
                self.inner.get(key)
            }
        }

        fn get_auth(&self, key: &str) -> Result<Option<String>, env::VarError> {
            if key == self.invalid_key {
                Err(env::VarError::NotUnicode("private-value-sentinel".into()))
            } else {
                Ok(self.inner.get(key))
            }
        }
    }

    const AUTH_FIELDS: [&str; 5] = [
        "FLOWL_EXTERNAL_URL",
        "FLOWL_OIDC_ISSUER",
        "FLOWL_OIDC_CLIENT_ID",
        "FLOWL_OIDC_CLIENT_SECRET",
        "FLOWL_OIDC_PROVIDER_NAME",
    ];

    #[test]
    fn non_unicode_auth_values_fail_closed_without_exposing_values() {
        for key in std::iter::once("FLOWL_AUTH_ENABLED").chain(AUTH_FIELDS) {
            let source = NonUnicodeConfig {
                inner: enabled_config(),
                invalid_key: key,
            };
            let error = AuthConfig::load_from(&source)
                .err()
                .expect("must reject invalid Unicode");
            assert_eq!(
                error,
                AuthConfigError::NonUnicodeValue(key.strip_prefix("FLOWL_").unwrap().to_owned())
            );
            assert!(error.to_string().contains(key));
            assert!(!error.to_string().contains("private-value-sentinel"));
            assert!(!format!("{error:?}").contains("private-value-sentinel"));
        }
    }

    #[test]
    fn disabled_auth_never_reads_other_auth_fields() {
        struct DisabledSource(Option<&'static str>);
        impl ConfigSource for DisabledSource {
            fn get(&self, _: &str) -> Option<String> {
                panic!("auth parser must use get_auth");
            }
            fn get_auth(&self, key: &str) -> Result<Option<String>, env::VarError> {
                assert_eq!(key, "FLOWL_AUTH_ENABLED");
                Ok(self.0.map(str::to_string))
            }
        }
        for flag in [None, Some("false")] {
            assert!(matches!(
                AuthConfig::load_from(&DisabledSource(flag)),
                Ok(AuthConfig::Disabled)
            ));
            for key in AUTH_FIELDS {
                let mut inner = enabled_config();
                inner.0.remove("FLOWL_AUTH_ENABLED");
                if let Some(flag) = flag {
                    inner = inner.with("FLOWL_AUTH_ENABLED", flag);
                }
                let source = NonUnicodeConfig {
                    inner,
                    invalid_key: key,
                };
                assert!(matches!(
                    AuthConfig::load_from(&source),
                    Ok(AuthConfig::Disabled)
                ));
            }
        }
    }

    #[test]
    fn enabled_auth_keeps_ordinary_configuration_lenient() {
        for key in [
            "FLOWL_PORT",
            "FLOWL_DB_PATH",
            "FLOWL_LOG_LEVEL",
            "FLOWL_MQTT_HOST",
            "FLOWL_MQTT_PORT",
            "FLOWL_MQTT_TOPIC_PREFIX",
            "FLOWL_MQTT_DISABLED",
            "FLOWL_AI_API_KEY",
            "FLOWL_AI_BASE_URL",
            "FLOWL_AI_MODEL",
            "FLOWL_AI_RATE_LIMIT",
        ] {
            let source = NonUnicodeConfig {
                inner: enabled_config(),
                invalid_key: key,
            };
            assert!(AuthConfig::load_from(&source).is_ok());
            let config = Config::load_from(&source);
            assert_eq!(config.port, 4100);
            assert_eq!(config.db_path, "/data/flowl.db");
            assert_eq!(config.log_level, "info");
            assert_eq!(config.mqtt_host, "localhost");
            assert_eq!(config.mqtt_port, 1883);
            assert_eq!(config.mqtt_topic_prefix, "flowl");
            assert!(!config.mqtt_disabled);
            assert!(config.ai_api_key.is_none());
            assert_eq!(config.ai_base_url, "https://api.openai.com/v1");
            assert_eq!(config.ai_model, "gpt-4.1-mini");
            assert_eq!(config.ai_rate_limit, 10);
        }
        let source = enabled_config()
            .with("FLOWL_PORT", "invalid")
            .with("FLOWL_MQTT_PORT", "invalid")
            .with("FLOWL_MQTT_DISABLED", "invalid")
            .with("FLOWL_AI_RATE_LIMIT", "invalid");
        assert!(AuthConfig::load_from(&source).is_ok());
        let config = Config::load_from(&source);
        assert_eq!(config.port, 4100);
        assert_eq!(config.mqtt_port, 1883);
        assert!(!config.mqtt_disabled);
        assert_eq!(config.ai_rate_limit, 10);
    }

    fn enabled_config() -> MockConfig {
        MockConfig::new()
            .with("FLOWL_AUTH_ENABLED", "true")
            .with("FLOWL_EXTERNAL_URL", "https://flowl.example")
            .with("FLOWL_OIDC_ISSUER", "https://issuer.example/oidc")
            .with("FLOWL_OIDC_CLIENT_ID", "flowl")
            .with("FLOWL_OIDC_CLIENT_SECRET", "secret")
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

    #[test]
    fn auth_is_disabled_when_absent_or_false() {
        assert!(matches!(
            AuthConfig::load_from(&MockConfig::new()),
            Ok(AuthConfig::Disabled)
        ));
        assert!(matches!(
            AuthConfig::load_from(
                &MockConfig::new()
                    .with("FLOWL_AUTH_ENABLED", "false")
                    .with("FLOWL_EXTERNAL_URL", "http://not-validated.example")
            ),
            Ok(AuthConfig::Disabled)
        ));
    }

    #[test]
    fn auth_enabled_flag_is_strict() {
        assert!(matches!(
            AuthConfig::load_from(&MockConfig::new().with("FLOWL_AUTH_ENABLED", "true")),
            Err(AuthConfigError::MissingRequiredValue(key)) if key == "EXTERNAL_URL"
        ));
        for value in ["", "1", "TRUE", "yes", " false "] {
            assert!(matches!(
                AuthConfig::load_from(&MockConfig::new().with("FLOWL_AUTH_ENABLED", value)),
                Err(AuthConfigError::InvalidEnabledFlag)
            ));
        }
    }

    #[test]
    fn enabled_auth_requires_each_value_to_be_present_and_nonempty() {
        for key in [
            "FLOWL_EXTERNAL_URL",
            "FLOWL_OIDC_ISSUER",
            "FLOWL_OIDC_CLIENT_ID",
            "FLOWL_OIDC_CLIENT_SECRET",
        ] {
            let source = enabled_config();
            let values = source.0;
            let missing = MockConfig(
                values
                    .into_iter()
                    .filter(|(candidate, _)| candidate != key)
                    .collect(),
            );
            assert!(matches!(
                AuthConfig::load_from(&missing),
                Err(AuthConfigError::MissingRequiredValue(candidate)) if candidate == key.strip_prefix("FLOWL_").unwrap()
            ));

            let empty = enabled_config().with(key, " \t");
            assert!(matches!(
                AuthConfig::load_from(&empty),
                Err(AuthConfigError::MissingRequiredValue(candidate)) if candidate == key.strip_prefix("FLOWL_").unwrap()
            ));
        }
    }

    #[test]
    fn external_url_rejects_unsafe_forms() {
        for value in [
            "not a url",
            "http://flowl.example",
            "http://localhost:4100",
            "http://127.0.0.1:4100",
            "http://[::1]:4100",
            "https://user:pass@flowl.example",
            "https://flowl.example?debug=1",
            "https://flowl.example#fragment",
            "https://flowl.example/flowl",
        ] {
            let source = enabled_config().with("FLOWL_EXTERNAL_URL", value);
            assert!(matches!(
                AuthConfig::load_from(&source),
                Err(AuthConfigError::InvalidExternalUrl)
            ));
        }
    }

    #[test]
    fn issuer_rejects_unsafe_forms_but_allows_a_path() {
        for value in [
            "not a url",
            "http://issuer.example",
            "http://localhost:8080/issuer",
            "http://127.0.0.1:8080/issuer",
            "http://[::1]:8080/issuer",
            "https://user:pass@issuer.example",
            "https://issuer.example?tenant=flowl",
            "https://issuer.example#fragment",
        ] {
            let source = enabled_config().with("FLOWL_OIDC_ISSUER", value);
            assert!(matches!(
                AuthConfig::load_from(&source),
                Err(AuthConfigError::InvalidIssuerUrl)
            ));
        }

        assert!(matches!(
            AuthConfig::load_from(&enabled_config()),
            Ok(AuthConfig::Enabled(_))
        ));
    }

    #[test]
    fn provider_name_defaults_trims_and_has_a_scalar_bound() {
        let AuthConfig::Enabled(config) = AuthConfig::load_from(&enabled_config()).unwrap() else {
            panic!("expected enabled configuration");
        };
        assert_eq!(config.provider_name, "OpenID Connect");

        let custom = enabled_config().with("FLOWL_OIDC_PROVIDER_NAME", " \tCompany Login\n ");
        let AuthConfig::Enabled(config) = AuthConfig::load_from(&custom).unwrap() else {
            panic!("expected enabled configuration");
        };
        assert_eq!(config.provider_name, "Company Login");

        for value in [
            "",
            " ",
            "bad\nname",
            "bad\0name",
            "bad\u{0085}name",
            "bad\u{009f}name",
        ] {
            let source = enabled_config().with("FLOWL_OIDC_PROVIDER_NAME", value);
            assert!(matches!(
                AuthConfig::load_from(&source),
                Err(AuthConfigError::InvalidProviderName)
            ));
        }

        for scalar in ["a", "é", "🌱"] {
            let source = enabled_config().with("FLOWL_OIDC_PROVIDER_NAME", scalar.repeat(80));
            assert!(AuthConfig::load_from(&source).is_ok());
            let source = enabled_config().with("FLOWL_OIDC_PROVIDER_NAME", scalar.repeat(81));
            assert!(matches!(
                AuthConfig::load_from(&source),
                Err(AuthConfigError::InvalidProviderName)
            ));
        }
    }

    #[test]
    fn issuer_retains_its_exact_raw_string_for_future_discovery_and_token_checks() {
        let source = enabled_config().with("FLOWL_OIDC_ISSUER", "https://issuer.example");
        let AuthConfig::Enabled(config) = AuthConfig::load_from(&source).unwrap() else {
            panic!("expected enabled configuration");
        };
        assert_eq!(config.issuer, "https://issuer.example");
        assert_ne!(config.issuer, "https://issuer.example/");
    }

    #[test]
    fn callback_is_fixed_from_external_url_not_request_headers() {
        let AuthConfig::Enabled(config) = AuthConfig::load_from(&enabled_config()).unwrap() else {
            panic!("expected enabled configuration");
        };
        let mut headers = HeaderMap::new();
        headers.insert("host", "attacker.example".parse().unwrap());
        headers.insert(
            "forwarded",
            "host=attacker.example;proto=http".parse().unwrap(),
        );
        headers.insert("x-forwarded-host", "attacker.example".parse().unwrap());
        headers.insert("x-forwarded-proto", "http".parse().unwrap());

        assert!(!headers.is_empty());
        assert_eq!(
            config.callback_url.as_str(),
            "https://flowl.example/auth/callback"
        );
    }
}
