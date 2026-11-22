#![forbid(unsafe_code)]
// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vaulten_lib::run();
}
