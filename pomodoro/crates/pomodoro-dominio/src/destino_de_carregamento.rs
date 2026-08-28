use url::Url;

/// Pra onde carregar a URL de uma atividade no webview (PRD §7.2): video do
/// YouTube pede o wrapper local com iframe — navegar direto pra URL de
/// embed do YouTube falha com erro 153 (o player exige contexto de iframe
/// com referer real, que uma navegacao de topo nao tem). Demais URLs
/// carregam direto, como o site inteiro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinoDeCarregamento {
    Video(Url),
    Direto(Url),
}
