use pomodoro_dominio::{Atividade, Etapa};

/// PRD §8: so ao fim de um Foco, e so se a pausa que comeca tem atividade
/// com URL aplicavel, a janela principal deve roubar o foco de volta. Sem
/// URL o timer normal aparece sem nenhum efeito — nao ha nada pra "trazer
/// pra frente". Fim de Pausa nunca dispara nada aqui.
pub fn deve_tomar_foco(
    etapa_antes: Etapa,
    etapa_depois: Etapa,
    atividade_da_pausa: &Atividade,
) -> bool {
    etapa_antes == Etapa::Foco
        && etapa_depois == Etapa::Pausa
        && atividade_da_pausa.tem_url_aplicavel()
}

#[cfg(test)]
mod testes {
    use super::*;
    use pomodoro_dominio::UrlDeAtividade;

    fn atividade_com_url() -> Atividade {
        let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
        Atividade::nova(None, Some(url))
    }

    #[test]
    fn fim_de_foco_com_url_deve_tomar_foco() {
        assert!(deve_tomar_foco(
            Etapa::Foco,
            Etapa::Pausa,
            &atividade_com_url()
        ));
    }

    #[test]
    fn fim_de_foco_sem_url_nao_deve_tomar_foco() {
        assert!(!deve_tomar_foco(
            Etapa::Foco,
            Etapa::Pausa,
            &Atividade::vazia()
        ));
    }

    #[test]
    fn fim_de_pausa_nao_deve_tomar_foco() {
        assert!(!deve_tomar_foco(
            Etapa::Pausa,
            Etapa::Foco,
            &atividade_com_url()
        ));
    }

    #[test]
    fn foco_para_foco_nao_deve_tomar_foco() {
        assert!(!deve_tomar_foco(
            Etapa::Foco,
            Etapa::Foco,
            &atividade_com_url()
        ));
    }

    #[test]
    fn pausa_para_pausa_nao_deve_tomar_foco() {
        assert!(!deve_tomar_foco(
            Etapa::Pausa,
            Etapa::Pausa,
            &atividade_com_url()
        ));
    }
}
