use thiserror::Error;

use crate::DestinoDeCarregamento;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UrlDeAtividadeInvalida {
    #[error("url de atividade invalida: '{recebido}' nao e uma url")]
    NaoEUmaUrl { recebido: String },
    #[error("esquema de url nao permitido: '{esquema}', esperado http ou https")]
    EsquemaNaoPermitido { esquema: String },
}

/// URL de uma atividade de pausa (PRD §7.1). Validada no Rust — validacao no
/// frontend e so UX, nao seguranca. Apenas `http`/`https` passam; `file:`,
/// `javascript:`, `data:` e demais esquemas sao rejeitados nomeando o
/// esquema recebido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlDeAtividade(url::Url);

impl UrlDeAtividade {
    pub fn nova(bruta: &str) -> Result<Self, UrlDeAtividadeInvalida> {
        let url = url::Url::parse(bruta).map_err(|_| UrlDeAtividadeInvalida::NaoEUmaUrl {
            recebido: bruta.to_string(),
        })?;
        let esquema = url.scheme();
        if esquema != "http" && esquema != "https" {
            return Err(UrlDeAtividadeInvalida::EsquemaNaoPermitido {
                esquema: esquema.to_string(),
            });
        }
        Ok(Self(url))
    }

    pub fn como_str(&self) -> &str {
        self.0.as_str()
    }

    /// A `url::Url` ja validada — pro child webview isolado da atividade
    /// (PRD §7.2), que precisa do tipo, nao so da string.
    pub fn url(&self) -> &url::Url {
        &self.0
    }

    /// Hostname sem o prefixo "www." — rotulo de fallback quando a atividade
    /// nao tem nome (PRD §7.1).
    pub fn hostname_sem_www(&self) -> Option<&str> {
        self.0
            .host_str()
            .map(|host| host.strip_prefix("www.").unwrap_or(host))
    }

    /// Pra onde carregar essa URL no webview da atividade (PRD §7.2): video
    /// do YouTube/YouTube Music vira `Video` (wrapper local com iframe, ver
    /// [`DestinoDeCarregamento`]). Demais URLs (ex.: site de leitura) viram
    /// `Direto`, carregando como estao.
    pub fn destino_de_carregamento(&self) -> DestinoDeCarregamento {
        match crate::video_incorporado::url_incorporada(&self.0) {
            Some(embed) => DestinoDeCarregamento::Video(embed),
            None => DestinoDeCarregamento::Direto(self.0.clone()),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn http_e_https_sao_validos() {
        assert!(UrlDeAtividade::nova("http://exemplo.com").is_ok());
        assert!(UrlDeAtividade::nova("https://exemplo.com").is_ok());
    }

    #[test]
    fn esquema_file_e_rejeitado_nomeando_o_esquema() {
        let erro = UrlDeAtividade::nova("file:///etc/passwd").expect_err("teste");
        assert_eq!(
            erro,
            UrlDeAtividadeInvalida::EsquemaNaoPermitido {
                esquema: "file".to_string()
            }
        );
    }

    #[test]
    fn esquema_javascript_e_rejeitado_nomeando_o_esquema() {
        let erro = UrlDeAtividade::nova("javascript:alert(1)").expect_err("teste");
        assert_eq!(
            erro,
            UrlDeAtividadeInvalida::EsquemaNaoPermitido {
                esquema: "javascript".to_string()
            }
        );
    }

    #[test]
    fn esquema_data_e_rejeitado_nomeando_o_esquema() {
        let erro = UrlDeAtividade::nova("data:text/plain,oi").expect_err("teste");
        assert_eq!(
            erro,
            UrlDeAtividadeInvalida::EsquemaNaoPermitido {
                esquema: "data".to_string()
            }
        );
    }

    #[test]
    fn string_sem_forma_de_url_e_invalida() {
        let erro = UrlDeAtividade::nova("nao e uma url").expect_err("teste");
        assert_eq!(
            erro,
            UrlDeAtividadeInvalida::NaoEUmaUrl {
                recebido: "nao e uma url".to_string()
            }
        );
    }

    #[test]
    fn hostname_sem_www_remove_apenas_o_prefixo() {
        let url = UrlDeAtividade::nova("https://www.exemplo.com/pagina").expect("teste");
        assert_eq!(url.hostname_sem_www(), Some("exemplo.com"));
    }

    #[test]
    fn hostname_sem_www_preserva_host_sem_o_prefixo() {
        let url = UrlDeAtividade::nova("https://musica.exemplo.com").expect("teste");
        assert_eq!(url.hostname_sem_www(), Some("musica.exemplo.com"));
    }

    #[test]
    fn destino_de_carregamento_de_video_do_youtube_e_video_com_embed() {
        let url =
            UrlDeAtividade::nova("https://www.youtube.com/watch?v=dQw4w9WgXcQ").expect("teste");
        assert_eq!(
            url.destino_de_carregamento(),
            DestinoDeCarregamento::Video(
                url::Url::parse("https://www.youtube.com/embed/dQw4w9WgXcQ").expect("teste")
            )
        );
    }

    #[test]
    fn destino_de_carregamento_de_site_de_leitura_e_direto() {
        let url = UrlDeAtividade::nova("https://akitaonrails.com/algum-post").expect("teste");
        assert_eq!(
            url.destino_de_carregamento(),
            DestinoDeCarregamento::Direto(url.url().clone())
        );
    }
}
