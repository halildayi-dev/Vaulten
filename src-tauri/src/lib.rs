#![forbid(unsafe_code)]

/// Starts the Vaulten desktop application.
///
/// No Tauri plugins are enabled and no IPC commands are registered yet.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| panic!("error while running Vaulten: {err}"));
}
