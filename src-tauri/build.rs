fn main() {
    // Every application command must be registered here so that Tauri generates
    // an explicit `allow-<command>` permission. No commands exist yet (SETUP-001).
    let attributes = tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[]),
    );
    tauri_build::try_build(attributes).expect("failed to run Tauri build script");
}
