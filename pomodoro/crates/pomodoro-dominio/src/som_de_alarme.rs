/// Som de alarme tocado nas transicoes de etapa de uma sessao sem URL de
/// atividade aplicavel (PRD §7): quem tem URL ja usa o bloqueio em video
/// como sinal, entao nao precisa de som. Preferencia pura — nao afeta
/// nenhuma regra do dominio, so qual asset o frontend reproduz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SomDeAlarme {
    #[default]
    Classico,
    Suave,
    Urgente,
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn padrao_e_classico() {
        assert_eq!(SomDeAlarme::default(), SomDeAlarme::Classico);
    }
}
