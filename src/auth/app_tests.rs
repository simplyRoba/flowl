use crate::auth::validate_return_to;

#[test]
fn raw_login_queries_preserve_serialization_and_reject_invalid_destinations() {
    use crate::auth::return_to::return_to_from_query;
    for query in [
        "return_to=a&return_to=b",
        "return_to=https%3A%2F%2Fattacker.example&return_to=%2Fplants",
        "return_to=%2F%2Fattacker.example",
        "return_to=%FF",
        "return_to=%GG",
        "return_to=%2Fuploads",
        "return_to=%2Fuploads%2Fphoto.jpg",
        "return_to=%2F%2561pi%2Fplants",
        "return_to=%2Fauth%2Fcallback",
    ] {
        assert_eq!(return_to_from_query(Some(query)), "/", "{query}");
    }
    let target = "/plants/42?label=a b&ratio=100%25#entry%20one";
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("return_to", target)
        .finish();
    assert_eq!(return_to_from_query(Some(&query)), target);
    assert_eq!(
        return_to_from_query(Some("%72eturn_to=%2Fapiary")),
        "/apiary"
    );
}

#[test]
fn config_payload_keeps_provider_name_as_json_data() {
    let provider_name = "<provider>&\\\"name";
    let mut config = super::OidcConfig::loopback_test("http://127.0.0.1", "http://127.0.0.1");
    config.provider_name = provider_name.to_owned();
    for auto_redirect in [false, true] {
        config.auto_redirect = auto_redirect;
        assert_eq!(
            super::config_payload(Some(&config)),
            serde_json::json!({
                "enabled": true, "provider_name": provider_name, "auto_redirect": auto_redirect,
            })
        );
    }
    assert_eq!(
        super::config_payload(None),
        serde_json::json!({ "enabled": false })
    );
}

#[test]
fn provider_diagnostics_are_fixed_categories_without_sensitive_values() {
    use crate::auth::runtime::OidcStartupError;
    for error in [
        OidcStartupError::HttpClient,
        OidcStartupError::InvalidIssuer,
        OidcStartupError::Discovery,
        OidcStartupError::UnsafeEndpoint("token"),
        OidcStartupError::MissingTokenEndpoint,
        OidcStartupError::MissingCodeFlow,
        OidcStartupError::UnsupportedClientAuthentication,
        OidcStartupError::UnusableSigningKeys,
        OidcStartupError::InvalidCallback,
    ] {
        let diagnostic = error.to_string();
        assert_ne!(diagnostic, "");
        assert!(
            !diagnostic
                .contains("secret-marker callback-code nonce verifier access-token response-body")
        );
        assert!(!diagnostic.contains("secret-marker"));
    }
    assert_eq!(
        format!("{:?}", super::CallbackError::Authentication),
        "Authentication"
    );
    assert_eq!(
        format!("{:?}", super::CallbackError::ProviderUnavailable),
        "ProviderUnavailable"
    );
    assert!(
        !crate::auth::CookieKeyError
            .to_string()
            .contains("secret-marker")
    );
}

#[test]
fn session_and_transaction_expiry_constants_are_bounded() {
    assert_eq!(
        crate::auth::transaction::TRANSACTION_TTL,
        time::Duration::minutes(5)
    );
    assert_eq!(
        crate::auth::session::AUTHENTICATED_TTL,
        time::Duration::hours(12)
    );
}

#[test]
fn accepts_local_paths_queries_and_fragments() {
    for target in [
        "/",
        "/plants/42",
        "/plants/42?tab=care",
        "/plants/42?tab=care#entry-7",
        "/search?q=snake%20plant",
    ] {
        assert_eq!(validate_return_to(Some(target)), target);
    }
}

#[test]
fn rejects_external_and_protocol_relative_targets() {
    for target in [
        "https://attacker.example/",
        "http://attacker.example/",
        "//attacker.example/",
        "///attacker.example/",
        r"/\\attacker.example/",
    ] {
        assert!(validate_return_to(Some(target)) == "/", "{target}");
    }
}

#[test]
fn rejects_controls_backslashes_and_malformed_escapes() {
    for target in [
        "/plants\\42",
        "/plants\n42",
        "/plants%00",
        "/plants%5c42",
        "/plants%",
        "/plants%2",
        "/plants%zz",
        "/plants%ff",
    ] {
        assert!(validate_return_to(Some(target)) == "/", "{target:?}");
    }
}

#[test]
fn rejects_encoded_separators_and_normalization_confusion() {
    for target in [
        "/plants%2f42",
        "/plants%3fnext=1",
        "/plants%23section",
        "/./plants",
        "/plants/../login",
        "/%2e/login",
        "/plants/%2e%2e/login",
        "/plants/%252e%252e/login",
    ] {
        assert!(validate_return_to(Some(target)) == "/", "{target}");
    }
}

#[test]
fn rejects_login_and_auth_routes_including_encoded_equivalents() {
    for target in [
        "/login",
        "/login/",
        "/login/reset",
        "/auth",
        "/auth/",
        "/auth/callback",
        "/log%69n",
        "/a%75th/callback",
    ] {
        assert!(validate_return_to(Some(target)) == "/", "{target}");
    }
}

#[test]
fn rejects_oversized_targets() {
    let target = format!(
        "/{}",
        "a".repeat(crate::auth::return_to::RETURN_TO_MAX_BYTES)
    );
    assert_eq!(validate_return_to(Some(&target)), "/");
}

#[test]
fn unsafe_values_fall_back_to_root() {
    assert_eq!(validate_return_to(Some("https://attacker.example/")), "/");
    assert_eq!(validate_return_to(Some("/plants/42")), "/plants/42");
}
