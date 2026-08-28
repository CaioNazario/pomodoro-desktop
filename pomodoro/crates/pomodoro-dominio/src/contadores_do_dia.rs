use crate::{Data, Duracao};

/// Contadores de um unico dia. `tempo_de_foco` soma o decorrido real em
/// Foco, inclusive o de uma sessao pulada pela metade — nunca
/// `sessoes_concluidas * duracao`, que mente quando ha pulo (ver PRD §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContadoresDoDia {
    dia_de_referencia: Data,
    sessoes_concluidas: u32,
    tempo_de_foco: Duracao,
    pausas_interrompidas: u32,
}

impl ContadoresDoDia {
    pub const fn vazio(dia: Data) -> Self {
        Self {
            dia_de_referencia: dia,
            sessoes_concluidas: 0,
            tempo_de_foco: Duracao::ZERO,
            pausas_interrompidas: 0,
        }
    }

    /// Reconstroi a partir de valores persistidos — usado por
    /// `pomodoro-config` ao carregar o TOML do disco.
    pub const fn reconstruir(
        dia: Data,
        sessoes_concluidas: u32,
        tempo_de_foco: Duracao,
        pausas_interrompidas: u32,
    ) -> Self {
        Self {
            dia_de_referencia: dia,
            sessoes_concluidas,
            tempo_de_foco,
            pausas_interrompidas,
        }
    }

    pub const fn dia_de_referencia(self) -> Data {
        self.dia_de_referencia
    }

    pub const fn sessoes_concluidas(self) -> u32 {
        self.sessoes_concluidas
    }

    pub const fn tempo_de_foco(self) -> Duracao {
        self.tempo_de_foco
    }

    pub const fn pausas_interrompidas(self) -> u32 {
        self.pausas_interrompidas
    }

    pub(crate) fn com_foco_somado(self, tempo_decorrido: Duracao) -> Self {
        Self {
            tempo_de_foco: self.tempo_de_foco.mais(tempo_decorrido),
            ..self
        }
    }

    pub(crate) fn com_sessao_concluida(self, tempo_decorrido: Duracao) -> Self {
        Self {
            sessoes_concluidas: self.sessoes_concluidas + 1,
            ..self.com_foco_somado(tempo_decorrido)
        }
    }

    pub(crate) fn com_pausa_interrompida(self) -> Self {
        Self {
            pausas_interrompidas: self.pausas_interrompidas + 1,
            ..self
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn dia_vazio_nao_tem_contadores() {
        let dia = ContadoresDoDia::vazio(Data::de(2026, 8, 23));
        assert_eq!(dia.sessoes_concluidas(), 0);
        assert_eq!(dia.tempo_de_foco(), Duracao::ZERO);
        assert_eq!(dia.pausas_interrompidas(), 0);
    }

    #[test]
    fn sessao_concluida_incrementa_contagem_e_soma_tempo() {
        let dia = ContadoresDoDia::vazio(Data::de(2026, 8, 23))
            .com_sessao_concluida(Duracao::de_minutos(25));
        assert_eq!(dia.sessoes_concluidas(), 1);
        assert_eq!(dia.tempo_de_foco(), Duracao::de_minutos(25));
    }

    #[test]
    fn foco_parcial_soma_tempo_sem_contar_sessao() {
        let dia =
            ContadoresDoDia::vazio(Data::de(2026, 8, 23)).com_foco_somado(Duracao::de_minutos(12));
        assert_eq!(dia.sessoes_concluidas(), 0);
        assert_eq!(dia.tempo_de_foco(), Duracao::de_minutos(12));
    }

    #[test]
    fn pausa_interrompida_incrementa_so_o_proprio_contador() {
        let dia = ContadoresDoDia::vazio(Data::de(2026, 8, 23)).com_pausa_interrompida();
        assert_eq!(dia.pausas_interrompidas(), 1);
        assert_eq!(dia.sessoes_concluidas(), 0);
    }
}
