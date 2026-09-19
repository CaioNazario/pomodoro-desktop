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

/// Mostra/esconde a janela da atividade sem sair do bloqueio (PRD §7.6) —
/// chamado pelo proprio `bloqueio.js` ao abrir/fechar o modal de
/// confirmacao da Urgencia, que senao renderiza atras da atividade.
#[tauri::command]
#[specta::specta]
pub fn definir_visibilidade_atividade(app: AppHandle, visivel: bool) {
    let _ = crate::webview_atividade::definir_visivel(&app, visivel);
}
