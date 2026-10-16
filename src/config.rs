use std::env;
use std::fmt;

use openidconnect::IssuerUrl;
use url::Url;

pub const DEFAULT_OIDC_PROVIDER_NAME: &str = "OpenID Connect";
pub const MAX_OIDC_PROVIDER_NAME_CHARS: usize = 80;

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
        let enabled = match auth_value(source, "FLOWL_AUTH_ENABLED")? {
            None => false,
            Some(value) => value
                .parse::<bool>()
                .map_err(|_| AuthConfigError::InvalidEnabledFlag)?,
        };

        if !enabled {
            return Ok(Self::Disabled);
        }

        let external_url_value = required(source, "FLOWL_EXTERNAL_URL")?;
        let external_url = ExternalUrl::parse(&external_url_value)?;
        let issuer_value = required(source, "FLOWL_OIDC_ISSUER")?;
        let issuer = IssuerConfig::parse(&issuer_value)?;
        let client_id = required(source, "FLOWL_OIDC_CLIENT_ID")?;
        let client_secret = required(source, "FLOWL_OIDC_CLIENT_SECRET")?;
        let provider_name = auth_value(source, "FLOWL_OIDC_PROVIDER_NAME")?.map_or_else(
            || Ok(DEFAULT_OIDC_PROVIDER_NAME.to_string()),
            |value| validate_provider_name(&value),
        )?;

        Ok(Self::Enabled(Box::new(EnabledAuthConfig {
            external_url,
            issuer,
            client_id,
            client_secret,
            provider_name,
        })))
    }

    pub fn enabled(&self) -> Option<&EnabledAuthConfig> {
        match self {
            Self::Disabled => None,
            Self::Enabled(config) => Some(config.as_ref()),
        }
    }
}

#[derive(Clone)]
pub struct EnabledAuthConfig {
    external_url: ExternalUrl,
    issuer: IssuerConfig,
    client_id: String,
    client_secret: String,
    provider_name: String,
}

impl EnabledAuthConfig {
    pub fn callback_url(&self) -> String {
        self.external_url.callback_url()
    }

    pub const fn issuer(&self) -> &IssuerConfig {
        &self.issuer
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    /// Constructs loopback-only HTTP configuration for in-crate OIDC tests. Production parsing
    /// always uses [`AuthConfig::load`] and therefore cannot select this policy.
    #[cfg(test)]
    pub(crate) fn loopback_test(issuer: &str, external_url: &str) -> Self {
        let issuer_url = IssuerUrl::new(issuer.to_string()).expect("test issuer URL is valid");
        Self {
            external_url: ExternalUrl(
                Url::parse(external_url).expect("test external URL is valid"),
            ),
            issuer: IssuerConfig {
                raw: issuer.to_string(),
                issuer_url,
            },
            client_id: "flowl-test-client".to_string(),
            client_secret: "flowl-test-secret".to_string(),
            provider_name: DEFAULT_OIDC_PROVIDER_NAME.to_string(),
        }
    }

    #[cfg(test)]
    pub(crate) fn permits_loopback_http(&self) -> bool {
        self.issuer.raw().parse::<Url>().is_ok_and(|url| {
            url.scheme() == "http" && url.host().is_some_and(|host| is_loopback_host(&host))
        })
    }
}

#[cfg(test)]
fn is_loopback_host(host: &url::Host<&str>) -> bool {
    matches!(host, url::Host::Domain("localhost"))
        || host
            .to_string()
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

#[derive(Clone)]
pub struct IssuerConfig {
    raw: String,
    issuer_url: IssuerUrl,
}

impl IssuerConfig {
    fn parse(raw: &str) -> Result<Self, AuthConfigError> {
        let url = Url::parse(raw).map_err(|_| AuthConfigError::InvalidIssuerUrl)?;
        if url.scheme() != "https"
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(AuthConfigError::InvalidIssuerUrl);
        }

        // Keep the configured spelling: `IssuerUrl` stores its original input separately from
        // its parsed URL, so never construct this from `Url::to_string()`.
        let issuer_url =
            IssuerUrl::new(raw.to_string()).map_err(|_| AuthConfigError::InvalidIssuerUrl)?;
        Ok(Self {
            raw: raw.to_string(),
            issuer_url,
        })
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub const fn issuer_url(&self) -> &IssuerUrl {
        &self.issuer_url
    }

    /// Compares a discovery or verified token issuer to the unmodified configured value.
    pub fn matches_raw(&self, issuer: &str) -> bool {
        issuer == self.raw
    }
}

#[derive(Clone)]
struct ExternalUrl(Url);

impl ExternalUrl {
    fn parse(value: &str) -> Result<Self, AuthConfigError> {
        let url = Url::parse(value).map_err(|_| AuthConfigError::InvalidExternalUrl)?;
        if url.scheme() != "https"
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(AuthConfigError::InvalidExternalUrl);
        }
        Ok(Self(url))
    }

    fn callback_url(&self) -> String {
        self.0
            .join("/auth/callback")
            .expect("validated HTTPS origin can join the fixed callback path")
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthConfigError {
    InvalidEnabledFlag,
    NonUnicodeValue(&'static str),
    MissingRequiredValue(&'static str),
    InvalidExternalUrl,
    InvalidIssuerUrl,
    InvalidProviderName,
}

impl fmt::Display for AuthConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEnabledFlag => formatter.write_str("FLOWL_AUTH_ENABLED must be a boolean"),
            Self::NonUnicodeValue(key) => write!(formatter, "{key} must be valid UTF-8"),
            Self::MissingRequiredValue(key) => write!(formatter, "{key} must be non-empty"),
            Self::InvalidExternalUrl => {
                formatter.write_str("FLOWL_EXTERNAL_URL must be an HTTPS origin")
            }
            Self::InvalidIssuerUrl => formatter.write_str("FLOWL_OIDC_ISSUER must be an HTTPS URL"),
            Self::InvalidProviderName => formatter.write_str("FLOWL_OIDC_PROVIDER_NAME is invalid"),
        }
    }
}

impl std::error::Error for AuthConfigError {}

fn auth_value(
    source: &impl ConfigSource,
    key: &'static str,
) -> Result<Option<String>, AuthConfigError> {
    source
        .get_auth(key)
        .map_err(|_| AuthConfigError::NonUnicodeValue(key))
}

fn required(source: &impl ConfigSource, key: &'static str) -> Result<String, AuthConfigError> {
    let value = auth_value(source, key)?.ok_or(AuthConfigError::MissingRequiredValue(key))?;
    if value.trim().is_empty() {
        return Err(AuthConfigError::MissingRequiredValue(key));
    }
    Ok(value)
}

fn validate_provider_name(value: &str) -> Result<String, AuthConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_control)
        || trimmed.chars().count() > MAX_OIDC_PROVIDER_NAME_CHARS
    {
        return Err(AuthConfigError::InvalidProviderName);
    }
    Ok(trimmed.to_string())
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
            assert_eq!(error, AuthConfigError::NonUnicodeValue(key));
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
            Err(AuthConfigError::MissingRequiredValue("FLOWL_EXTERNAL_URL"))
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
                Err(AuthConfigError::MissingRequiredValue(candidate)) if candidate == key
            ));

            let empty = enabled_config().with(key, " \t");
            assert!(matches!(
                AuthConfig::load_from(&empty),
                Err(AuthConfigError::MissingRequiredValue(candidate)) if candidate == key
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
        assert_eq!(config.provider_name(), DEFAULT_OIDC_PROVIDER_NAME);

        let custom = enabled_config().with("FLOWL_OIDC_PROVIDER_NAME", " \tCompany Login\n ");
        let AuthConfig::Enabled(config) = AuthConfig::load_from(&custom).unwrap() else {
            panic!("expected enabled configuration");
        };
        assert_eq!(config.provider_name(), "Company Login");

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
        let issuer = config.issuer();
        assert_eq!(issuer.raw(), "https://issuer.example");
        assert_eq!(issuer.issuer_url().as_str(), "https://issuer.example");
        assert!(!issuer.matches_raw("https://issuer.example/"));
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
        assert_eq!(config.callback_url(), "https://flowl.example/auth/callback");
    }
}
