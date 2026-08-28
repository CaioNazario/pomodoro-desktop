use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DuracaoInvalida {
    #[error("duracao negativa: {recebido}ms, esperado >= 0")]
    Negativa { recebido: i64 },
}

/// Instante absoluto em milissegundos desde a epoca Unix.
///
/// O dominio nao usa `SystemTime`: quem sabe a hora e o [`crate::Relogio`] injetado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instante(i64);

impl Instante {
    pub const fn desde_epoca_ms(milissegundos: i64) -> Self {
        Self(milissegundos)
    }

    pub const fn epoca_ms(self) -> i64 {
        self.0
    }

    pub const fn mais(self, duracao: Duracao) -> Self {
        Self(self.0 + duracao.0)
    }

    /// Quanto passou de `anterior` ate `self`. Nunca negativo: relogio que anda
    /// para tras vira duracao zero, nao panico.
    pub const fn decorrido_desde(self, anterior: Self) -> Duracao {
        let bruto = self.0 - anterior.0;
        Duracao(if bruto < 0 { 0 } else { bruto })
    }
}

/// Intervalo nao negativo, em milissegundos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Duracao(i64);

impl Duracao {
    pub const ZERO: Self = Self(0);

    pub const fn de_minutos(minutos: u16) -> Self {
        Self(minutos as i64 * 60_000)
    }

    pub fn de_ms(milissegundos: i64) -> Result<Self, DuracaoInvalida> {
        if milissegundos < 0 {
            return Err(DuracaoInvalida::Negativa {
                recebido: milissegundos,
            });
        }
        Ok(Self(milissegundos))
    }

    pub const fn em_ms(self) -> i64 {
        self.0
    }

    pub const fn esgotada(self) -> bool {
        self.0 == 0
    }

    pub const fn mais(self, outra: Self) -> Self {
        Self(self.0 + outra.0)
    }

    /// Subtracao saturando em zero — nunca negativa, mesmo se `outra` for
    /// maior que `self`.
    pub const fn menos(self, outra: Self) -> Self {
        let diferenca = self.0 - outra.0;
        Self(if diferenca < 0 { 0 } else { diferenca })
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn decorrido_conta_o_intervalo_entre_dois_instantes() {
        let inicio = Instante::desde_epoca_ms(1_000);
        let fim = Instante::desde_epoca_ms(4_500);
        assert_eq!(
            fim.decorrido_desde(inicio),
            Duracao::de_ms(3_500).expect("teste")
        );
    }

    #[test]
    fn relogio_andando_para_tras_produz_duracao_zero_e_nao_panico() {
        let inicio = Instante::desde_epoca_ms(9_000);
        let fim = Instante::desde_epoca_ms(1_000);
        assert_eq!(fim.decorrido_desde(inicio), Duracao::ZERO);
    }

    #[test]
    fn duracao_negativa_e_erro_que_nomeia_o_valor_ofensor() {
        let erro = Duracao::de_ms(-7).expect_err("teste");
        assert_eq!(erro, DuracaoInvalida::Negativa { recebido: -7 });
        assert_eq!(erro.to_string(), "duracao negativa: -7ms, esperado >= 0");
    }

    #[test]
    fn minutos_viram_milissegundos() {
        assert_eq!(Duracao::de_minutos(25).em_ms(), 1_500_000);
    }

    #[test]
    fn mais_soma_duas_duracoes() {
        assert_eq!(
            Duracao::de_minutos(3).mais(Duracao::de_minutos(4)),
            Duracao::de_minutos(7)
        );
    }

    #[test]
    fn menos_subtrai_sem_ir_abaixo_de_zero() {
        assert_eq!(
            Duracao::de_minutos(10).menos(Duracao::de_minutos(4)),
            Duracao::de_minutos(6)
        );
        assert_eq!(
            Duracao::de_minutos(4).menos(Duracao::de_minutos(10)),
            Duracao::ZERO
        );
    }
}
