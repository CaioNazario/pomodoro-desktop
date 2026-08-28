use crate::{Atividade, Duracao};

/// Duracoes de foco e pausa e atividade de uma sessao especifica do plano
/// individual (PRD §3). Uma sessao nova comeca com `Atividade::vazia()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sessao {
    foco: Duracao,
    pausa: Duracao,
    atividade: Atividade,
}

impl Sessao {
    pub fn nova(foco: Duracao, pausa: Duracao) -> Self {
        Self {
            foco,
            pausa,
            atividade: Atividade::vazia(),
        }
    }

    pub const fn foco(&self) -> Duracao {
        self.foco
    }

    pub const fn pausa(&self) -> Duracao {
        self.pausa
    }

    pub fn atividade(&self) -> &Atividade {
        &self.atividade
    }

    pub fn com_foco(self, nova: Duracao) -> Self {
        Self { foco: nova, ..self }
    }

    pub fn com_pausa(self, nova: Duracao) -> Self {
        Self {
            pausa: nova,
            ..self
        }
    }

    pub fn com_atividade(self, nova: Atividade) -> Self {
        Self {
            atividade: nova,
            ..self
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn nova_sessao_guarda_foco_e_pausa() {
        let sessao = Sessao::nova(Duracao::de_minutos(25), Duracao::de_minutos(5));
        assert_eq!(sessao.foco(), Duracao::de_minutos(25));
        assert_eq!(sessao.pausa(), Duracao::de_minutos(5));
    }

    #[test]
    fn com_foco_substitui_so_o_foco() {
        let sessao = Sessao::nova(Duracao::de_minutos(25), Duracao::de_minutos(5))
            .com_foco(Duracao::de_minutos(50));
        assert_eq!(sessao.foco(), Duracao::de_minutos(50));
        assert_eq!(sessao.pausa(), Duracao::de_minutos(5));
    }

    #[test]
    fn com_pausa_substitui_so_a_pausa() {
        let sessao = Sessao::nova(Duracao::de_minutos(25), Duracao::de_minutos(5))
            .com_pausa(Duracao::de_minutos(10));
        assert_eq!(sessao.foco(), Duracao::de_minutos(25));
        assert_eq!(sessao.pausa(), Duracao::de_minutos(10));
    }
}
