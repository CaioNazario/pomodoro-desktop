use tauri::{AppHandle, State};

use crate::bloqueio::GerenciadorDeBloqueio;

/// Unico command exposto a janela de bloqueio (PRD §7.6) — derruba o
/// webview da atividade, destroi as janelas de bloqueio, restaura os
/// atalhos do GNOME e conta como pausa interrompida. Nao toca no timer.
#[tauri::command]
#[specta::specta]
pub fn confirmar_urgencia(app: AppHandle, bloqueio: State<GerenciadorDeBloqueio>) {
    bloqueio.confirmar_urgencia(&app);
}
