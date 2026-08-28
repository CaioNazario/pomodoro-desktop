// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Forca XWayland antes de qualquer chamada GTK. GDK le `GDK_BACKEND` uma vez,
/// na primeira inicializacao — setar aqui cobre `cargo run`, `pnpm tauri dev`
/// e o binario final igual, sem depender de como o processo foi lancado.
/// Decisao arquitetural #5: Wayland puro nao tem `set_position` nem
/// always-on-top, e o widget/tomada-de-foco dependem dos dois.
#[cfg(target_os = "linux")]
fn forcar_xwayland() {
    std::env::set_var("GDK_BACKEND", "x11");
}

#[cfg(not(target_os = "linux"))]
fn forcar_xwayland() {}

fn main() {
    forcar_xwayland();
    pomodoro_lib::run()
}
