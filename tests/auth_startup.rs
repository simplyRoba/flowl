use std::process::Stdio;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::process::Command;

#[tokio::test]
async fn executable_reports_safe_discovery_transport_failure_before_startup() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("must-not-exist.db");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!(
        "https://{}/issuer-path-sentinel",
        listener.local_addr().unwrap()
    );
    drop(listener);

    let mut command = Command::new(env!("CARGO_BIN_EXE_flowl"));
    let child = command
        .current_dir(directory.path())
        .env_clear()
        .env("FLOWL_AUTH_ENABLED", "true")
        .env("FLOWL_EXTERNAL_URL", "https://localhost")
        .env("FLOWL_OIDC_ISSUER", &issuer)
        .env("FLOWL_OIDC_CLIENT_ID", "client-id-sentinel")
        .env("FLOWL_OIDC_CLIENT_SECRET", "client-secret-sentinel")
        .env("FLOWL_OIDC_PROVIDER_NAME", "provider-name-sentinel")
        .env("FLOWL_LOG_LEVEL", "info")
        .env("FLOWL_DB_PATH", &database)
        .env("FLOWL_MQTT_DISABLED", "true")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let output = tokio::time::timeout(Duration::from_secs(15), child.wait_with_output())
        .await
        .expect("startup must terminate promptly")
        .unwrap();
    assert!(!output.status.success());
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostics.contains("OIDC discovery failed"));
    assert!(diagnostics.contains("transport failure"));
    assert!(diagnostics.contains("Unable to initialize authentication"));
    for forbidden in [
        "client-id-sentinel",
        "client-secret-sentinel",
        "provider-name-sentinel",
        "issuer-path-sentinel",
        issuer.as_str(),
        "https://",
        database.to_str().unwrap(),
        "Starting flowl",
        "Database ready",
        "Listening",
    ] {
        assert!(!diagnostics.contains(forbidden), "unexpected {forbidden}");
    }
    assert!(!database.exists());
    assert!(!directory.path().join("uploads").exists());
}
