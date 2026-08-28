use crate::{Duracao, Instante};

/// Estado do timer de uma etapa. O prazo e sempre absoluto — nao existe
/// acumulador de tick. Suspensao do SO nao corrompe isto: o restante e
/// sempre `prazo - agora`, nunca `restante -= delta`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoTimer {
    Ocioso { restante: Duracao },
    Correndo { prazo: Instante },
    Pausado { restante: Duracao },
}

impl EstadoTimer {
    pub const fn ocioso(restante: Duracao) -> Self {
        Self::Ocioso { restante }
    }

    /// Ocioso/Pausado -> Correndo com prazo recalculado a partir de `agora`.
    /// Correndo -> Pausado guardando o restante no instante da pausa.
    pub fn alternar_execucao(self, agora: Instante) -> Self {
        match self {
            Self::Ocioso { restante } | Self::Pausado { restante } => Self::Correndo {
                prazo: agora.mais(restante),
            },
            Self::Correndo { prazo } => Self::Pausado {
                restante: prazo.decorrido_desde(agora),
            },
        }
    }

    pub const fn reiniciar(duracao_cheia: Duracao) -> Self {
        Self::Ocioso {
            restante: duracao_cheia,
        }
    }

    pub fn restante_em(self, agora: Instante) -> Duracao {
        match self {
            Self::Ocioso { restante } | Self::Pausado { restante } => restante,
            Self::Correndo { prazo } => prazo.decorrido_desde(agora),
        }
    }

    pub fn venceu_em(self, agora: Instante) -> bool {
        match self {
            Self::Correndo { prazo } => agora >= prazo,
            Self::Ocioso { .. } | Self::Pausado { .. } => false,
        }
    }

    pub const fn rodando(self) -> bool {
        matches!(self, Self::Correndo { .. })
    }

    /// Prazo absoluto, se estiver `Correndo`. Fronteira Rust<->TS usa isto
    /// para o React derivar `mm:ss` localmente sem receber tick.
    pub const fn prazo(self) -> Option<Instante> {
        match self {
            Self::Correndo { prazo } => Some(prazo),
            Self::Ocioso { .. } | Self::Pausado { .. } => None,
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::relogio_fake::RelogioFake;
    use crate::Relogio;

    #[test]
    fn alternar_execucao_de_ocioso_calcula_prazo_a_partir_de_agora() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        let timer = EstadoTimer::ocioso(Duracao::de_minutos(25)).alternar_execucao(relogio.agora());
        relogio.avancar(Duracao::de_minutos(10));
        assert_eq!(timer.restante_em(relogio.agora()), Duracao::de_minutos(15));
    }

    #[test]
    fn pausar_e_retomar_preserva_o_restante_e_recalcula_o_prazo() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        let timer = EstadoTimer::ocioso(Duracao::de_minutos(25)).alternar_execucao(relogio.agora());

        relogio.avancar(Duracao::de_minutos(10));
        let pausado = timer.alternar_execucao(relogio.agora());
        assert_eq!(
            pausado,
            EstadoTimer::Pausado {
                restante: Duracao::de_minutos(15)
            }
        );

        relogio.avancar(Duracao::de_minutos(3));
        let retomado = pausado.alternar_execucao(relogio.agora());
        assert_eq!(
            retomado,
            EstadoTimer::Correndo {
                prazo: Instante::desde_epoca_ms(13 * 60_000 + 15 * 60_000)
            }
        );
        assert_eq!(
            retomado.restante_em(relogio.agora()),
            Duracao::de_minutos(15)
        );
    }

    #[test]
    fn reiniciar_sempre_produz_ocioso_com_duracao_cheia() {
        assert_eq!(
            EstadoTimer::reiniciar(Duracao::de_minutos(25)),
            EstadoTimer::ocioso(Duracao::de_minutos(25))
        );
    }

    #[test]
    fn venceu_em_e_falso_fora_de_correndo() {
        assert!(!EstadoTimer::ocioso(Duracao::ZERO).venceu_em(Instante::desde_epoca_ms(0)));
        assert!(!EstadoTimer::Pausado {
            restante: Duracao::ZERO
        }
        .venceu_em(Instante::desde_epoca_ms(0)));
    }

    #[test]
    fn venceu_em_e_verdadeiro_quando_agora_alcanca_o_prazo() {
        let timer = EstadoTimer::Correndo {
            prazo: Instante::desde_epoca_ms(1_000),
        };
        assert!(!timer.venceu_em(Instante::desde_epoca_ms(999)));
        assert!(timer.venceu_em(Instante::desde_epoca_ms(1_000)));
        assert!(timer.venceu_em(Instante::desde_epoca_ms(5_000)));
    }

    #[test]
    fn prazo_so_existe_quando_correndo() {
        assert_eq!(EstadoTimer::ocioso(Duracao::ZERO).prazo(), None);
        let prazo = Instante::desde_epoca_ms(1_000);
        assert_eq!(EstadoTimer::Correndo { prazo }.prazo(), Some(prazo));
    }
}
