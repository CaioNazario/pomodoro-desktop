use pomodoro_dominio::Atividade;

/// PRD §7: toda transicao de etapa cuja pausa envolvida nao tem URL
/// aplicavel deve tocar o alarme sonoro — quem tem URL ja usa o bloqueio em
/// video como sinal, nao precisa de som. Foco->Foco nao existe (mesma
/// premissa de `tomada_de_foco::deve_tomar_foco`), entao toda transicao
/// real e Foco<->Pausa e passa por aqui.
pub fn deve_tocar_alarme(atividade_da_transicao: &Atividade) -> bool {
    !atividade_da_transicao.tem_url_aplicavel()
}

#[cfg(test)]
mod testes {
    use super::*;
    use pomodoro_dominio::UrlDeAtividade;

    #[test]
    fn sem_url_deve_tocar_alarme() {
        assert!(deve_tocar_alarme(&Atividade::vazia()));
    }

    #[test]
    fn com_url_nao_deve_tocar_alarme() {
        let url = UrlDeAtividade::nova("https://exemplo.com").expect("teste");
        assert!(!deve_tocar_alarme(&Atividade::nova(None, Some(url))));
    }
}
