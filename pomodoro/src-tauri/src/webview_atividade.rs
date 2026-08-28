use pomodoro_dominio::DestinoDeCarregamento;
use tauri::{LogicalPosition, Manager, Runtime, WebviewBuilder, WebviewUrl, Window};
use url::form_urlencoded;

/// Label do webview isolado da atividade de pausa (PRD §7.2). Ausente de todo
/// `capabilities/*.json` de proposito: sem isso, o conteudo carregado nao
/// alcanca nenhum command, evento ou API do Tauri. Verificado em
/// `tests/capabilities_atividade.rs`.
pub const LABEL: &str = "atividade";

/// Cria o webview da atividade como filho de `janela`, cobrindo toda a area
/// de cliente dela. Incognito, sem qualquer capability propria — navegar
/// dentro dele nao alcanca IPC nenhum. Posicionamento por monitor (uma
/// janela de bloqueio cada) e responsabilidade de quem chama, nao deste
/// modulo.
pub fn abrir<R: Runtime>(janela: &Window<R>, destino: &DestinoDeCarregamento) -> tauri::Result<()> {
    let tamanho = janela.inner_size()?;
    let builder = WebviewBuilder::new(LABEL, url_do_webview(destino)).incognito(true);
    janela.add_child(builder, LogicalPosition::new(0.0, 0.0), tamanho)?;
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

/// Derruba o webview da atividade, se existir. Sem efeito se o bloqueio nao
/// estiver ativo — chamado tanto pela Urgencia (§7.6) quanto ao sair do
/// bloqueio normalmente.
pub fn fechar<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let Some(webview) = app.get_webview(LABEL) else {
        return Ok(());
    };
    webview.close()
}
