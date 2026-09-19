use pomodoro_dominio::DestinoDeCarregamento;
use tauri::{Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::form_urlencoded;

/// Label do webview isolado da atividade de pausa (PRD §7.2). Ausente de todo
/// `capabilities/*.json` de proposito: sem isso, o conteudo carregado nao
/// alcanca nenhum command, evento ou API do Tauri. Verificado em
/// `tests/capabilities_atividade.rs`.
pub const LABEL: &str = "atividade";

/// Cria a atividade como janela top-level propria (nao webview filho),
/// cobrindo exatamente o retangulo do monitor de `janela_pai` (o mesmo
/// usado pra criar ela, ver `bloqueio::janela::criar_janela_de_bloqueio`).
///
/// Nao usa `add_child`: no Linux, Tauri encaixa todo webview filho
/// (`WebviewKind::WindowChild`) no mesmo `GtkBox` vertical do webview
/// principal da janela-pai — com os dois pedindo expand+fill, o GTK
/// divide o espaco igualmente entre eles, ignorando qualquer `bounds`
/// que a gente passe (medido ao vivo: tela sempre partida ao meio,
/// independente do tamanho/posicao enviados). Janela separada tem seu
/// proprio `GtkBox`, entao nao compete espaco com o `bloqueio.html`.
///
/// Sem capability propria: navegar dentro dela nao alcanca IPC nenhum.
/// Posicionamento por monitor (uma janela de bloqueio cada) e
/// responsabilidade de quem chama, nao deste modulo.
///
/// Nao incognito de proposito: o YouTube incorporado depende de
/// cookies/armazenamento de terceiros pra liberar a reproducao embutida —
/// em modo incognito ele so mostra o cartao "Assista no YouTube" em vez do
/// player (medido ao vivo).
pub fn abrir<R: Runtime>(
    janela_pai: &WebviewWindow<R>,
    destino: &DestinoDeCarregamento,
    posicao: PhysicalPosition<i32>,
    tamanho: PhysicalSize<u32>,
) -> tauri::Result<()> {
    WebviewWindowBuilder::new(janela_pai, LABEL, url_do_webview(destino))
        .parent(janela_pai)?
        .position(posicao.x as f64, posicao.y as f64)
        .inner_size(tamanho.width as f64, tamanho.height as f64)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build()?;
    Ok(())
}

/// Video pede o wrapper local `atividade-video.html` com o embed dentro de
/// um `<iframe>` — navegar direto pra URL de embed do YouTube falha com
/// erro 153, o player exige contexto de iframe com referer real (PRD §7.2).
/// URL direta carrega como esta, o site inteiro.
fn url_do_webview(destino: &DestinoDeCarregamento) -> WebviewUrl {
    match destino {
        DestinoDeCarregamento::Video(embed) => {
            let src: String = form_urlencoded::byte_serialize(embed.as_str().as_bytes()).collect();
            WebviewUrl::App(format!("atividade-video.html?src={src}").into())
        }
        DestinoDeCarregamento::Direto(url) => WebviewUrl::External(url.clone()),
    }
}

/// Derruba a janela da atividade, se existir. Sem efeito se o bloqueio nao
/// estiver ativo — chamado tanto pela Urgencia (§7.6) quanto ao sair do
/// bloqueio normalmente.
pub fn fechar<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let Some(janela) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    janela.close()
}

/// Mostra/esconde a janela da atividade sem destrui-la. Usado pelo command
/// `definir_visibilidade_atividade`, chamado do proprio `bloqueio.js` ao
/// abrir/fechar o modal de confirmacao da Urgencia (PRD §7.6): o modal e
/// centralizado na janela de bloqueio, que fica atras da atividade — sem
/// esconder a atividade, o modal renderiza mas nunca aparece na tela.
/// Sem efeito se a atividade nao existir (pausa sem URL aplicavel).
pub fn definir_visivel<R: Runtime>(app: &tauri::AppHandle<R>, visivel: bool) -> tauri::Result<()> {
    let Some(janela) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    if visivel {
        janela.show()
    } else {
        janela.hide()
    }
}
