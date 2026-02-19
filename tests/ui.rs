use std::process::Command;

#[test]
fn ui_tests() {
    let ui_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");

    if !ui_dir.join("node_modules").exists() {
        let status = Command::new("npm")
            .arg("install")
            .current_dir(&ui_dir)
            .status()
            .expect("failed to run npm install");
        assert!(status.success(), "npm install failed");
    }

    let output = Command::new("npm")
        .args(["run", "test"])
        .current_dir(&ui_dir)
        .output()
        .expect("failed to run npm test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "UI tests failed:\n{stdout}\n{stderr}"
    );
}
