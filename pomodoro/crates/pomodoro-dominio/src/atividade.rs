use crate::UrlDeAtividade;

const ROTULO_PADRAO: &str = "atividade";

/// Atividade de pausa (PRD §3, §7.1): nome e URL independentemente
/// opcionais. Em modo Global existe uma so, repetida em todas as pausas; em
/// modo Individual, uma por sessao — quem decide isso e [`crate::PlanoDoCiclo`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Atividade {
    nome: Option<String>,
    url: Option<UrlDeAtividade>,
}

impl Atividade {
    pub fn vazia() -> Self {
        Self::default()
    }

    pub fn nova(nome: Option<String>, url: Option<UrlDeAtividade>) -> Self {
        Self { nome, url }
    }

    pub fn nome(&self) -> Option<&str> {
        self.nome.as_deref()
    }

    pub fn url(&self) -> Option<&UrlDeAtividade> {
        self.url.as_ref()
    }

    /// Ha URL aplicavel para essa atividade — condicao de entrada em
    /// bloqueio ao avancar para uma Pausa (PRD §4.2.5).
    pub fn tem_url_aplicavel(&self) -> bool {
        self.url.is_some()
    }

    /// Rotulo exibido (PRD §7.1): o nome; na falta dele, o hostname sem
    /// "www."; na falta dos dois, "atividade".
    pub fn rotulo(&self) -> String {
        self.nome
            .clone()
            .or_else(|| {
                self.url
                    .as_ref()
                    .and_then(UrlDeAtividade::hostname_sem_www)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| ROTULO_PADRAO.to_string())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn atividade_vazia_nao_tem_url_aplicavel() {
        assert!(!Atividade::vazia().tem_url_aplicavel());
    }

    #[test]
    fn atividade_com_url_tem_url_aplicavel() {
        let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
        let atividade = Atividade::nova(None, Some(url));
        assert!(atividade.tem_url_aplicavel());
    }

    #[test]
    fn rotulo_usa_o_nome_quando_presente() {
        let url = UrlDeAtividade::nova("https://www.exemplo.com").expect("teste");
        let atividade = Atividade::nova(Some("Musica pra focar".to_string()), Some(url));
        assert_eq!(atividade.rotulo(), "Musica pra focar");
    }

    #[test]
    fn rotulo_cai_para_o_hostname_sem_nome() {
        let url = UrlDeAtividade::nova("https://www.exemplo.com/pagina").expect("teste");
        let atividade = Atividade::nova(None, Some(url));
        assert_eq!(atividade.rotulo(), "exemplo.com");
    }

    #[test]
    fn rotulo_cai_para_o_padrao_sem_nome_e_sem_url() {
        assert_eq!(Atividade::vazia().rotulo(), "atividade");
    }
}
