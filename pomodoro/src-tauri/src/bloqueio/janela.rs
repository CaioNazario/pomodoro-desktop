use tauri::webview::WebviewWindowBuilder;
use tauri::window::Monitor;
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WindowEvent};

/// Uma janela de bloqueio por monitor (PRD §7.3): sem decoracao,
/// always-on-top, fullscreen cobrindo exatamente o retangulo daquele
/// monitor. `CloseRequested` e recusado — a unica saida e a Urgencia
/// (fatia 5, parte 5), nunca o botao/atalho de fechar do SO.
pub(super) fn criar_janela_de_bloqueio(
    app: &AppHandle,
    label: &str,
    monitor: &Monitor,
    script_de_inicializacao: &str,
) -> tauri::Result<WebviewWindow> {
    let posicao = monitor.position();
    let tamanho = monitor.size();

    // `fullscreen` fica FORA do builder de proposito: com `.fullscreen(true)`
    // antes do `.build()`, no GTK3 + webkit2gtk-4.1 sob mutter/XWayland a
    // janela nao mapeia — `build()` devolve `Ok`, `is_visible()` devolve
    // `Ok(true)`, e nenhuma janela X11 chega a existir. Criada no retangulo
    // do monitor (`position` + `inner_size`) e promovida a fullscreen depois
    // que ja existe, mapeia normalmente. Medido, nao suposto.
    let janela = WebviewWindowBuilder::new(app, label, WebviewUrl::App("bloqueio.html".into()))
        .initialization_script(script_de_inicializacao)
        .position(posicao.x as f64, posicao.y as f64)
        .inner_size(tamanho.width as f64, tamanho.height as f64)
        .decorations(false)
        .always_on_top(true)
        .build()?;
    janela.set_fullscreen(true)?;

    let app_para_eventos = app.clone();
    janela.on_window_event(move |evento| match evento {
        WindowEvent::CloseRequested { api, .. } => api.prevent_close(),
        WindowEvent::Focused(false) => super::fuga::ao_perder_foco(&app_para_eventos),
        _ => {}
    });

    Ok(janela)
}
