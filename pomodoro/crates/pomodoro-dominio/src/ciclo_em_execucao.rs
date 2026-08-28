use crate::{
    ciclo::NumeroDeSessao, quantidade_de_sessoes::QuantidadeDeSessoes, timer::EstadoTimer, Duracao,
    Etapa, Instante,
};

/// Agregado do ciclo: etapa + sessao + timer + duracoes globais.
///
/// Nome deliberado: "sessao" ja e o par foco+pausa no glossario do produto,
/// entao o agregado nao se chama `Sessao` para nao confundir os dois.
///
/// `duracao_foco`/`duracao_pausa` guardam a duracao *ativa* no momento —
/// quem decide se isso e o valor global ou o de uma sessao especifica do
/// plano individual e o `PlanoDoCiclo`, que envolve este tipo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CicloEmExecucao {
    etapa: Etapa,
    sessao: NumeroDeSessao,
    total_sessoes: QuantidadeDeSessoes,
    timer: EstadoTimer,
    duracao_foco: Duracao,
    duracao_pausa: Duracao,
    iniciar_automaticamente: bool,
}

impl CicloEmExecucao {
    pub fn novo(
        total_sessoes: QuantidadeDeSessoes,
        duracao_foco: Duracao,
        duracao_pausa: Duracao,
        iniciar_automaticamente: bool,
    ) -> Self {
        Self {
            etapa: Etapa::Foco,
            sessao: NumeroDeSessao::primeira(),
            total_sessoes,
            timer: EstadoTimer::ocioso(duracao_foco),
            duracao_foco,
            duracao_pausa,
            iniciar_automaticamente,
        }
    }

    pub const fn etapa(&self) -> Etapa {
        self.etapa
    }

    pub const fn sessao(&self) -> NumeroDeSessao {
        self.sessao
    }

    /// Devolve o total cru: o contrato Rust<->TS e o resto do dominio nao
    /// precisam do newtype validado, so da construcao pra frente.
    pub const fn total_sessoes(&self) -> u8 {
        self.total_sessoes.valor()
    }

    pub const fn rodando(&self) -> bool {
        self.timer.rodando()
    }

    pub const fn iniciar_automaticamente(&self) -> bool {
        self.iniciar_automaticamente
    }

    /// Troca so a preferencia de auto-start, sem tocar etapa/sessao/timer em curso.
    pub fn com_iniciar_automaticamente(self, valor: bool) -> Self {
        Self {
            iniciar_automaticamente: valor,
            ..self
        }
    }

    pub fn restante_em(&self, agora: Instante) -> Duracao {
        self.timer.restante_em(agora)
    }

    pub fn venceu_em(&self, agora: Instante) -> bool {
        self.timer.venceu_em(agora)
    }

    pub const fn prazo(&self) -> Option<Instante> {
        self.timer.prazo()
    }

    pub const fn duracao_da_etapa_atual(&self) -> Duracao {
        match self.etapa {
            Etapa::Foco => self.duracao_foco,
            Etapa::Pausa => self.duracao_pausa,
        }
    }

    /// Ocioso/Pausado -> Correndo; Correndo -> Pausado. Ver [`EstadoTimer::alternar_execucao`].
    pub fn alternar_execucao(self, agora: Instante) -> Self {
        Self {
            timer: self.timer.alternar_execucao(agora),
            ..self
        }
    }

    /// Volta a etapa atual para Ocioso com a duracao cheia.
    pub fn reiniciar_etapa(self) -> Self {
        Self {
            timer: EstadoTimer::reiniciar(self.duracao_da_etapa_atual()),
            ..self
        }
    }

    /// Alterna a etapa. A sessao so incrementa ao entrar em Foco, circularmente.
    /// Auto-start assimetrico: entrando em Pausa sempre inicia; entrando em
    /// Foco respeita `iniciar_automaticamente`. Chamada uma unica vez tanto
    /// por `pular_etapa` quanto pelo vencimento do prazo — retomar apos uma
    /// suspensao longa do SO nao encadeia etapas, porque o novo prazo nasce
    /// de `agora`, nunca do prazo antigo.
    pub fn avancar(self, agora: Instante) -> Self {
        let proxima_etapa = self.etapa.alternar();
        let proxima_sessao = match proxima_etapa {
            Etapa::Foco => self.sessao.proxima(self.total_sessoes),
            Etapa::Pausa => self.sessao,
        };
        let duracao_destino = match proxima_etapa {
            Etapa::Foco => self.duracao_foco,
            Etapa::Pausa => self.duracao_pausa,
        };
        let deve_rodar = match proxima_etapa {
            Etapa::Pausa => true,
            Etapa::Foco => self.iniciar_automaticamente,
        };
        let timer = if deve_rodar {
            EstadoTimer::Correndo {
                prazo: agora.mais(duracao_destino),
            }
        } else {
            EstadoTimer::ocioso(duracao_destino)
        };

        Self {
            etapa: proxima_etapa,
            sessao: proxima_sessao,
            timer,
            ..self
        }
    }

    /// Atualiza a duracao de foco guardada. Se a etapa atual for Foco,
    /// tambem realinha o timer (ver [`Self::timer_realinhado`]); caso
    /// contrario so troca o valor guardado, sem tocar no timer corrente.
    pub fn com_duracao_foco(self, nova: Duracao, agora: Instante) -> Self {
        let etapa_ativa = self.etapa == Etapa::Foco;
        let timer = if etapa_ativa {
            Self::timer_realinhado(self.timer, nova, agora)
        } else {
            self.timer
        };
        Self {
            duracao_foco: nova,
            timer,
            ..self
        }
    }

    /// Espelho de [`Self::com_duracao_foco`] para a pausa.
    pub fn com_duracao_pausa(self, nova: Duracao, agora: Instante) -> Self {
        let etapa_ativa = self.etapa == Etapa::Pausa;
        let timer = if etapa_ativa {
            Self::timer_realinhado(self.timer, nova, agora)
        } else {
            self.timer
        };
        Self {
            duracao_pausa: nova,
            timer,
            ..self
        }
    }

    /// Se `rodando`, recalcula o prazo como `agora + nova`. Se parado,
    /// recarrega o restante para a duracao cheia nova.
    fn timer_realinhado(timer: EstadoTimer, nova: Duracao, agora: Instante) -> EstadoTimer {
        if timer.rodando() {
            EstadoTimer::Correndo {
                prazo: agora.mais(nova),
            }
        } else {
            EstadoTimer::ocioso(nova)
        }
    }

    /// Troca total de sessoes e numero da sessao em curso sem tocar no
    /// timer — usado pelo redimensionamento, que nunca altera o prazo
    /// corrente.
    pub fn com_total_e_sessao(self, total: QuantidadeDeSessoes, sessao: NumeroDeSessao) -> Self {
        Self {
            total_sessoes: total,
            sessao,
            ..self
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::relogio_fake::RelogioFake;
    use crate::Relogio;

    fn ciclo_de_quatro(iniciar_automaticamente: bool) -> CicloEmExecucao {
        CicloEmExecucao::novo(
            QuantidadeDeSessoes::nova(4).expect("teste"),
            Duracao::de_minutos(25),
            Duracao::de_minutos(5),
            iniciar_automaticamente,
        )
    }

    #[test]
    fn iniciar_automaticamente_reflete_o_valor_construido() {
        assert!(ciclo_de_quatro(true).iniciar_automaticamente());
        assert!(!ciclo_de_quatro(false).iniciar_automaticamente());
    }

    #[test]
    fn com_iniciar_automaticamente_troca_so_essa_flag() {
        let ciclo = ciclo_de_quatro(true).com_iniciar_automaticamente(false);
        assert!(!ciclo.iniciar_automaticamente());
        assert_eq!(ciclo.etapa(), Etapa::Foco);
        assert_eq!(ciclo.sessao().valor(), 1);
        assert!(!ciclo.rodando());
    }

    #[test]
    fn foco_avanca_para_pausa_sem_incrementar_sessao() {
        let ciclo = ciclo_de_quatro(true).avancar(Instante::desde_epoca_ms(0));
        assert_eq!(ciclo.etapa(), Etapa::Pausa);
        assert_eq!(ciclo.sessao().valor(), 1);
    }

    #[test]
    fn pausa_avanca_para_foco_incrementando_sessao() {
        let ciclo = ciclo_de_quatro(true)
            .avancar(Instante::desde_epoca_ms(0))
            .avancar(Instante::desde_epoca_ms(0));
        assert_eq!(ciclo.etapa(), Etapa::Foco);
        assert_eq!(ciclo.sessao().valor(), 2);
    }

    #[test]
    fn ciclo_circular_volta_da_sessao_quatro_para_a_um() {
        let mut ciclo = ciclo_de_quatro(true);
        let agora = Instante::desde_epoca_ms(0);
        for _ in 0..(4 * 2) {
            ciclo = ciclo.avancar(agora);
        }
        assert_eq!(ciclo.etapa(), Etapa::Foco);
        assert_eq!(ciclo.sessao().valor(), 1);
    }

    #[test]
    fn entrar_em_pausa_sempre_inicia_mesmo_com_autostart_desligado() {
        let ciclo = ciclo_de_quatro(false).avancar(Instante::desde_epoca_ms(0));
        assert_eq!(ciclo.etapa(), Etapa::Pausa);
        assert!(ciclo.rodando());
    }

    #[test]
    fn entrar_em_foco_respeita_autostart_desligado() {
        let ciclo = ciclo_de_quatro(false)
            .avancar(Instante::desde_epoca_ms(0))
            .avancar(Instante::desde_epoca_ms(0));
        assert_eq!(ciclo.etapa(), Etapa::Foco);
        assert!(!ciclo.rodando());
    }

    #[test]
    fn entrar_em_foco_inicia_quando_autostart_ligado() {
        let ciclo = ciclo_de_quatro(true)
            .avancar(Instante::desde_epoca_ms(0))
            .avancar(Instante::desde_epoca_ms(0));
        assert_eq!(ciclo.etapa(), Etapa::Foco);
        assert!(ciclo.rodando());
    }

    #[test]
    fn pausar_e_retomar_preserva_o_restante() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        let ciclo = ciclo_de_quatro(true).alternar_execucao(relogio.agora());
        relogio.avancar(Duracao::de_minutos(10));

        let pausado = ciclo.alternar_execucao(relogio.agora());
        assert!(!pausado.rodando());
        assert_eq!(
            pausado.restante_em(relogio.agora()),
            Duracao::de_minutos(15)
        );

        relogio.avancar(Duracao::de_minutos(3));
        let retomado = pausado.alternar_execucao(relogio.agora());
        assert!(retomado.rodando());
        assert_eq!(
            retomado.restante_em(relogio.agora()),
            Duracao::de_minutos(15)
        );
    }

    #[test]
    fn retomada_apos_suspensao_avanca_uma_unica_etapa() {
        let ciclo = ciclo_de_quatro(true).alternar_execucao(Instante::desde_epoca_ms(0));
        let prazo_original = Instante::desde_epoca_ms(0).mais(Duracao::de_minutos(25));

        // dormiu 3h alem do prazo do foco
        let acordar = prazo_original.mais(Duracao::de_minutos(180));
        assert!(ciclo.venceu_em(acordar));

        let depois = ciclo.avancar(acordar);
        assert_eq!(depois.etapa(), Etapa::Pausa);
        assert_eq!(depois.sessao().valor(), 1);
        // o novo prazo nasce de `acordar`, nao de um encadeamento de etapas
        assert_eq!(depois.restante_em(acordar), Duracao::de_minutos(5));
    }

    #[test]
    fn reiniciar_etapa_recarrega_a_duracao_cheia_da_etapa_atual() {
        let ciclo = ciclo_de_quatro(true).alternar_execucao(Instante::desde_epoca_ms(0));
        let reiniciado = ciclo.reiniciar_etapa();
        assert!(!reiniciado.rodando());
        assert_eq!(
            reiniciado.restante_em(Instante::desde_epoca_ms(0)),
            Duracao::de_minutos(25)
        );
    }

    #[test]
    fn alterar_duracao_da_etapa_ativa_rodando_recalcula_o_prazo_a_partir_de_agora() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        let ciclo = ciclo_de_quatro(true).alternar_execucao(relogio.agora());
        relogio.avancar(Duracao::de_minutos(12));

        let alterado = ciclo.com_duracao_foco(Duracao::de_minutos(50), relogio.agora());
        assert!(alterado.rodando());
        assert_eq!(
            alterado.restante_em(relogio.agora()),
            Duracao::de_minutos(50)
        );
    }

    #[test]
    fn alterar_duracao_da_etapa_ativa_parada_recarrega_o_restante_para_a_duracao_cheia() {
        let ciclo = ciclo_de_quatro(false);
        assert!(!ciclo.rodando());

        let alterado = ciclo.com_duracao_foco(Duracao::de_minutos(50), Instante::desde_epoca_ms(0));
        assert!(!alterado.rodando());
        assert_eq!(
            alterado.restante_em(Instante::desde_epoca_ms(0)),
            Duracao::de_minutos(50)
        );
    }

    #[test]
    fn alterar_duracao_que_nao_e_a_ativa_so_atualiza_o_valor_guardado() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        let ciclo = ciclo_de_quatro(true).alternar_execucao(relogio.agora());
        relogio.avancar(Duracao::de_minutos(12));

        // etapa atual e Foco: alterar a duracao da PAUSA nao mexe no timer.
        let alterado = ciclo.com_duracao_pausa(Duracao::de_minutos(20), relogio.agora());
        assert!(alterado.rodando());
        assert_eq!(
            alterado.restante_em(relogio.agora()),
            Duracao::de_minutos(13)
        );
        assert_eq!(alterado.duracao_pausa, Duracao::de_minutos(20));
    }

    #[test]
    fn com_total_e_sessao_nao_toca_no_timer() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        let ciclo = ciclo_de_quatro(true).alternar_execucao(relogio.agora());
        relogio.avancar(Duracao::de_minutos(10));

        let redimensionado = ciclo.com_total_e_sessao(
            QuantidadeDeSessoes::nova(3).expect("teste"),
            NumeroDeSessao::de(3),
        );
        assert_eq!(redimensionado.total_sessoes(), 3);
        assert_eq!(redimensionado.sessao().valor(), 3);
        assert!(redimensionado.rodando());
        assert_eq!(
            redimensionado.restante_em(relogio.agora()),
            Duracao::de_minutos(15)
        );
    }
}
