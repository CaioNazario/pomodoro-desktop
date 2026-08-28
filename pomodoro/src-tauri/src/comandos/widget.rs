use tauri::AppHandle;

/// Alterna mostrar/esconder a janela do widget (PRD §9) — chamado pelo
/// botao "Widget" no cabecalho da janela principal.
#[tauri::command]
#[specta::specta]
pub fn alternar_widget(app: AppHandle) {
    crate::widget::alternar_visibilidade(&app);
}
