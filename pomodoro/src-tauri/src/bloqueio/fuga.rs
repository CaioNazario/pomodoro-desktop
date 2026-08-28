use std::time::Duration;

use tauri::{AppHandle, Manager};

use super::GerenciadorDeBloqueio;

/// Espera antes de decidir que houve fuga — cobre o caso comum e inofensivo
/// de o foco migrar de uma janela de bloqueio pra outra (usuario olhando o
/// segundo monitor), que dispara `Focused(false)` numa e `Focused(true)` na
/// outra quase simultaneamente. Curto o bastante pra nao ser perceptivel
/// como delay de reacao a uma fuga de verdade.
const ATRASO_DE_VERIFICACAO: Duration = Duration::from_millis(150);

/// Chamado pelo handler de `WindowEvent::Focused(false)` de toda janela de
/// bloqueio (PRD §7.4). Roda numa thread separada porque o handler de evento
/// do Tauri nao pode bloquear a fila de eventos da janela.
pub(super) fn ao_perder_foco(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(ATRASO_DE_VERIFICACAO);
        verificar_fuga(&app);
    });
}

fn verificar_fuga(app: &AppHandle) {
    let Some(gerenciador) = app.try_state::<GerenciadorDeBloqueio>() else {
        return;
    };
    let Ok(janelas) = gerenciador.janelas_ativas.lock() else {
        return;
    };
    if janelas.is_empty() {
        return; // bloqueio ja nao esta ativo
    }
    let alguma_das_nossas_esta_focada = janelas.iter().any(|label| {
        app.get_webview_window(label)
            .and_then(|janela| janela.is_focused().ok())
            .unwrap_or(false)
    });
    let labels: Vec<String> = janelas.clone();
    drop(janelas);

    if alguma_das_nossas_esta_focada {
        return;
    }
    reagir_a_fuga(app, &labels);
}

/// PRD §7.4, passos 1-3. O passo 4 (mostrar a confirmacao de Urgencia
/// pulada ao recuperar o foco) fica pra quando essa UI existir (fatia 5,
/// parte 5) — nao ha o que mostrar ainda.
fn reagir_a_fuga(app: &AppHandle, labels: &[String]) {
    for label in labels {
        if let Some(janela) = app.get_webview_window(label) {
            let _ = janela.set_fullscreen(true);
            let _ = janela.set_focus();
        }
    }
    crate::registro_de_foco::registrar_pausa_interrompida(app);
    crate::persistencia::persistir(app);
}
