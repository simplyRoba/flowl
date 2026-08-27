use std::path::Path;
use std::process::Command;

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create UI build directory");
    for entry in std::fs::read_dir(source).expect("read UI static directory") {
        let entry = entry.expect("read UI static entry");
        let destination = destination.join(entry.file_name());
        if entry
            .file_type()
            .expect("read UI static entry type")
            .is_dir()
        {
            copy_directory(&entry.path(), &destination);
        } else {
            std::fs::copy(entry.path(), destination).expect("copy UI static asset");
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/static");
    println!("cargo:rerun-if-changed=ui/svelte.config.js");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-env-changed=SKIP_UI_BUILD");

    if std::env::var("SKIP_UI_BUILD").is_ok() {
        let build_dir = Path::new("ui/build");
        if build_dir.exists() {
            std::fs::remove_dir_all(build_dir).expect("remove stale UI build");
        }
        copy_directory(Path::new("ui/static"), build_dir);
        std::fs::write(
            build_dir.join("index.html"),
            "<h1>Binary was compiled with SKIP_UI_BUILD</h1>",
        )
        .expect("write skipped UI index");
        std::fs::write(build_dir.join("service-worker.js"), "// SKIP_UI_BUILD stub")
            .expect("write skipped UI service worker");
        let immutable_dir = build_dir.join("_app/immutable");
        std::fs::create_dir_all(&immutable_dir).expect("create skipped immutable asset directory");
        std::fs::write(
            immutable_dir.join("skip-ui-build.js"),
            "// SKIP_UI_BUILD stub",
        )
        .expect("write skipped immutable asset");
        return;
    }

    let ui_dir = Path::new("ui");
    let build_dir = ui_dir.join("build");

    // Clean previous build (removes stub from SKIP_UI_BUILD if present)
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir).unwrap();
    }

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
