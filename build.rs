use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/static");
    println!("cargo:rerun-if-changed=ui/svelte.config.js");
    println!("cargo:rerun-if-changed=ui/package.json");

    let ui_dir = std::path::Path::new("ui");

    // Install dependencies if node_modules is missing
    if !ui_dir.join("node_modules").exists() {
        let status = Command::new("npm")
            .arg("install")
            .current_dir(ui_dir)
            .status()
            .expect("failed to run npm install");
        assert!(status.success(), "npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .status()
        .expect("failed to run npm build");
    assert!(status.success(), "SvelteKit build failed");
}
