use crate::{ContadoresDoDia, Data, Duracao};
use std::collections::VecDeque;

/// Ultimos 90 dias de contadores, em anel: ao ultrapassar a capacidade, o
/// dia mais antigo e descartado. A virada de dia e implicita — sempre que
/// `hoje` difere do ultimo dia registrado, um novo dia nasce.
const CAPACIDADE_EM_DIAS: usize = 90;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricoDiario {
    dias: VecDeque<ContadoresDoDia>,
}

impl HistoricoDiario {
    pub fn vazio() -> Self {
        Self {
            dias: VecDeque::new(),
        }
    }

    /// Reconstroi a partir de dias persistidos, mais antigo primeiro.
    /// Se vierem mais que 90, os mais antigos sao descartados.
    pub fn reconstruir(dias: Vec<ContadoresDoDia>) -> Self {
        let mut dias: VecDeque<ContadoresDoDia> = dias.into();
        while dias.len() > CAPACIDADE_EM_DIAS {
            dias.pop_front();
        }
        Self { dias }
    }

    /// Dias guardados, mais antigo primeiro — para persistencia.
    pub fn dias(&self) -> impl Iterator<Item = ContadoresDoDia> + '_ {
        self.dias.iter().copied()
    }

    /// Contadores de `hoje`, sem mutar o historico. Se ainda nao houver
    /// entrada pra `hoje`, devolve um dia vazio (nao registrado). Busca a
    /// partir do fim porque a data pedida costuma ser recente.
    pub fn dia_atual(&self, hoje: Data) -> ContadoresDoDia {
        self.dias
            .iter()
            .rev()
            .find(|dia| dia.dia_de_referencia() == hoje)
            .copied()
            .unwrap_or(ContadoresDoDia::vazio(hoje))
    }

    pub fn registrar_sessao_concluida(self, hoje: Data, tempo_decorrido: Duracao) -> Self {
        self.com_dia_atualizado(hoje, |dia| dia.com_sessao_concluida(tempo_decorrido))
    }

    pub fn registrar_foco_parcial(self, hoje: Data, tempo_decorrido: Duracao) -> Self {
        self.com_dia_atualizado(hoje, |dia| dia.com_foco_somado(tempo_decorrido))
    }

    pub fn registrar_pausa_interrompida(self, hoje: Data) -> Self {
        self.com_dia_atualizado(hoje, ContadoresDoDia::com_pausa_interrompida)
    }

    fn com_dia_atualizado(
        mut self,
        hoje: Data,
        atualizar: impl FnOnce(ContadoresDoDia) -> ContadoresDoDia,
    ) -> Self {
        let precisa_de_dia_novo = self
            .dias
            .back()
            .is_none_or(|dia| dia.dia_de_referencia() != hoje);
        if precisa_de_dia_novo {
            self.dias.push_back(ContadoresDoDia::vazio(hoje));
        }
        if self.dias.len() > CAPACIDADE_EM_DIAS {
            self.dias.pop_front();
        }
        let indice_do_ultimo = self.dias.len() - 1;
        self.dias[indice_do_ultimo] = atualizar(self.dias[indice_do_ultimo]);
        self
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::relogio_fake::RelogioFake;
    use crate::{Instante, Relogio};

    #[test]
    fn dia_atual_sem_registro_devolve_vazio() {
        let historico = HistoricoDiario::vazio();
        let hoje = Data::de(2026, 8, 23);
        assert_eq!(historico.dia_atual(hoje), ContadoresDoDia::vazio(hoje));
    }

    #[test]
    fn registrar_sessao_concluida_cria_o_dia_e_incrementa() {
        let hoje = Data::de(2026, 8, 23);
        let historico =
            HistoricoDiario::vazio().registrar_sessao_concluida(hoje, Duracao::de_minutos(25));
        let dia = historico.dia_atual(hoje);
        assert_eq!(dia.sessoes_concluidas(), 1);
        assert_eq!(dia.tempo_de_foco(), Duracao::de_minutos(25));
    }

    #[test]
    fn registrar_foco_parcial_nao_incrementa_sessao() {
        let hoje = Data::de(2026, 8, 23);
        let historico =
            HistoricoDiario::vazio().registrar_foco_parcial(hoje, Duracao::de_minutos(12));
        let dia = historico.dia_atual(hoje);
        assert_eq!(dia.sessoes_concluidas(), 0);
        assert_eq!(dia.tempo_de_foco(), Duracao::de_minutos(12));
    }

    #[test]
    fn virada_de_dia_com_relogio_fake_preserva_o_dia_anterior() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        relogio.mudar_dia(Data::de(2026, 8, 22));
        let historico = HistoricoDiario::vazio()
            .registrar_sessao_concluida(relogio.hoje(), Duracao::de_minutos(25));

        relogio.mudar_dia(Data::de(2026, 8, 23));
        let historico =
            historico.registrar_sessao_concluida(relogio.hoje(), Duracao::de_minutos(25));

        let ontem = historico.dia_atual(Data::de(2026, 8, 22));
        let hoje = historico.dia_atual(Data::de(2026, 8, 23));
        assert_eq!(ontem.sessoes_concluidas(), 1);
        assert_eq!(hoje.sessoes_concluidas(), 1);
        assert_eq!(historico.dias().count(), 2);
    }

    #[test]
    fn anel_de_90_dias_descarta_o_mais_antigo_ao_transbordar() {
        let mut historico = HistoricoDiario::vazio();
        for dia in 1..=91u8 {
            historico = historico.registrar_sessao_concluida(Data::de(2026, 1, dia), Duracao::ZERO);
        }
        assert_eq!(historico.dias().count(), 90);
        assert!(historico
            .dias()
            .all(|d| d.dia_de_referencia() != Data::de(2026, 1, 1)));
        assert_eq!(
            historico
                .dia_atual(Data::de(2026, 1, 91))
                .sessoes_concluidas(),
            1
        );
    }

    #[test]
    fn reconstruir_alem_da_capacidade_descarta_os_mais_antigos() {
        let dias: Vec<_> = (1..=95u8)
            .map(|dia| ContadoresDoDia::vazio(Data::de(2026, 1, dia)))
            .collect();
        let historico = HistoricoDiario::reconstruir(dias);
        assert_eq!(historico.dias().count(), 90);
        assert_eq!(
            historico.dias().next().map(|d| d.dia_de_referencia()),
            Some(Data::de(2026, 1, 6))
        );
    }
}
