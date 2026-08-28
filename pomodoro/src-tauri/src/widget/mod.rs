mod rastreador;

use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use rastreador::RastreadorDeArrasto;

pub const LABEL: &str = "widget";

/// Largura do design canvas (262px). A altura NAO e 150px como o design
/// pede — achado ao vivo: `webkit2gtk`/GTK impoe um minimo implicito de
/// ~200px por dimensao nesta plataforma, e `min_inner_size`/`resizable`
/// nao o removem. A janela nasce fixa nesse piso; o card interno (150px) e
/// a pilula (38px) sao renderizados dentro dela sem redimensionar a janela
/// — ver `Widget.tsx`.
const LARGURA: f64 = 262.0;
const ALTURA: f64 = 200.0;

/// Espelha o inset do design canvas (`popupX`/`popupY`, linhas 561-562 do
/// `.dc.html`): a folga extra alem do proprio tamanho do widget (26px/84px)
/// evita colar nas bordas da tela.
const MARGEM_DIREITA: f64 = 288.0;
const MARGEM_INFERIOR: f64 = 234.0;
const POSICAO_PADRAO_SEM_MONITOR: (i32, i32) = (24, 24);

const ATRASO_DE_PERSISTENCIA: Duration = Duration::from_millis(400);

/// Estado gerenciado do Tauri: a posicao corrente (pra `persistencia`
/// reconstruir `Configuracao`), o rastreador que debounça a escrita em
/// disco durante um arrasto continuo, e a visibilidade que o widget tinha
/// antes do bloqueio comecar (`None` quando o bloqueio nao esta ativo) —
/// pra `restaurar_apos_bloqueio` nao forcar o widget visivel se o usuario
/// ja tinha escondido ele antes.
pub struct EstadoDoWidget {
    posicao: Mutex<Option<(i32, i32)>>,
    rastreador: RastreadorDeArrasto,
    visivel_antes_do_bloqueio: Mutex<Option<bool>>,
}

impl EstadoDoWidget {
    pub fn novo(posicao_persistida: Option<(i32, i32)>) -> Self {
        Self {
            posicao: Mutex::new(posicao_persistida),
            rastreador: RastreadorDeArrasto::novo(),
            visivel_antes_do_bloqueio: Mutex::new(None),
        }
    }

    pub fn posicao_atual(&self) -> Option<(i32, i32)> {
        self.posicao.lock().ok().and_then(|guarda| *guarda)
    }
}

/// Cria a janela do widget (PRD §9): sem decoracao, always-on-top,
/// transparente (pro estado minimizado, uma pilula, nao aparecer como um
/// retangulo preto ao redor), posicionada no ultimo lugar salvo ou no canto
/// inferior direito do monitor primario. **Nao** e chamada no `setup()` —
/// o widget comeca fechado; so nasce lazy, na primeira chamada de
/// `alternar_visibilidade` (botao "Widget" do cabecalho).
pub fn abrir(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(LABEL).is_some() {
        return Ok(());
    }
    let estado = app.state::<EstadoDoWidget>();
    let (x, y) = estado
        .posicao_atual()
        .unwrap_or_else(|| posicao_padrao(app));

    let janela = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("widget.html".into()))
        .position(x as f64, y as f64)
        .inner_size(LARGURA, ALTURA)
        .decorations(false)
        .always_on_top(true)
        .resizable(false)
        .transparent(true)
        .skip_taskbar(true)
        .build()?;

    let app_para_eventos = app.clone();
    janela.on_window_event(move |evento| {
        if let WindowEvent::Moved(posicao) = evento {
            reagir_ao_mover(&app_para_eventos, (posicao.x, posicao.y));
        }
    });

    Ok(())
}

/// Alterna mostrar/esconder o widget — chamado pelo botao "Widget" do
/// cabecalho da janela principal. "Nao existe" cobre dois casos: o app
/// acabou de abrir e o widget ainda nao nasceu (`abrir` nunca rodou), ou o
/// usuario fechou pelo X da propria janela do widget (que **fecha**, nao
/// esconde). Nos dois casos este toggle deve criar/reabrir a janela, senao
/// o botao "Widget" viraria um no-op permanente.
pub fn alternar_visibilidade(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(LABEL) else {
        if let Err(erro) = abrir(app) {
            eprintln!("pomodoro: falha ao reabrir o widget: {erro}");
        }
        return;
    };
    let visivel = janela.is_visible().unwrap_or(true);
    let _ = if visivel {
        janela.hide()
    } else {
        janela.show()
    };
}

/// PRD §9: "durante o bloqueio o widget e escondido — a moldura ja mostra
/// o tempo restante". Chamado por `bloqueio::GerenciadorDeBloqueio::entrar`.
/// Guarda a visibilidade de antes pra `restaurar_apos_bloqueio` decidir
/// certo, inclusive se o usuario ja tinha escondido o widget manualmente.
pub fn esconder_para_bloqueio(app: &AppHandle) {
    let Some(janela) = app.get_webview_window(LABEL) else {
        return;
    };
    let visivel = janela.is_visible().unwrap_or(true);
    if let Some(estado) = app.try_state::<EstadoDoWidget>() {
        if let Ok(mut guarda) = estado.visivel_antes_do_bloqueio.lock() {
            *guarda = Some(visivel);
        }
    }
    let _ = janela.hide();
}

/// Contraparte de `esconder_para_bloqueio`, chamada por
/// `bloqueio::GerenciadorDeBloqueio::sair`. So mostra de volta se o widget
/// estava visivel antes do bloqueio comecar.
pub fn restaurar_apos_bloqueio(app: &AppHandle) {
    let Some(estado) = app.try_state::<EstadoDoWidget>() else {
        return;
    };
    let Ok(mut guarda) = estado.visivel_antes_do_bloqueio.lock() else {
        return;
    };
    let estava_visivel = guarda.take().unwrap_or(true);
    if !estava_visivel {
        return;
    }
    if let Some(janela) = app.get_webview_window(LABEL) {
        let _ = janela.show();
    }
}

fn posicao_padrao(app: &AppHandle) -> (i32, i32) {
    let tamanho_do_monitor = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|monitor| monitor.size().to_owned())
        .map(|tamanho| (tamanho.width, tamanho.height));
    posicao_padrao_para(tamanho_do_monitor)
}

fn posicao_padrao_para(tamanho_do_monitor: Option<(u32, u32)>) -> (i32, i32) {
    let Some((largura, altura)) = tamanho_do_monitor else {
        return POSICAO_PADRAO_SEM_MONITOR;
    };
    (
        (largura as f64 - MARGEM_DIREITA).max(0.0) as i32,
        (altura as f64 - MARGEM_INFERIOR).max(0.0) as i32,
    )
}

/// Grava a posicao em memoria imediatamente, mas so persiste em disco apos
/// `ATRASO_DE_PERSISTENCIA` sem nenhum movimento novo (ver
/// `RastreadorDeArrasto`) — mesma familia de tecnica do debounce de fuga em
/// `bloqueio::fuga`, adaptada de "atraso fixo" pra "e a ultima geracao".
fn reagir_ao_mover(app: &AppHandle, posicao: (i32, i32)) {
    let Some(estado) = app.try_state::<EstadoDoWidget>() else {
        return;
    };
    let Ok(mut guarda) = estado.posicao.lock() else {
        return;
    };
    *guarda = Some(posicao);
    drop(guarda);

    let minha_geracao = estado.rastreador.registrar_movimento();
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(ATRASO_DE_PERSISTENCIA);
        let Some(estado) = app.try_state::<EstadoDoWidget>() else {
            return;
        };
        if estado.rastreador.ainda_e_a_ultima(minha_geracao) {
            crate::persistencia::persistir(&app);
        }
    });
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn sem_monitor_usa_a_posicao_padrao_fixa() {
        assert_eq!(posicao_padrao_para(None), POSICAO_PADRAO_SEM_MONITOR);
    }

    #[test]
    fn com_monitor_aplica_a_margem_do_design_canvas() {
        assert_eq!(posicao_padrao_para(Some((1920, 1080))), (1632, 846));
    }

    #[test]
    fn monitor_menor_que_a_margem_nao_fica_negativo() {
        assert_eq!(posicao_padrao_para(Some((200, 200))), (0, 0));
    }
}
