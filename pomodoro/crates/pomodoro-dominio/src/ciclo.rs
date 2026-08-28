use crate::QuantidadeDeSessoes;

/// Numero da sessao dentro do ciclo, 1-based. Circular sobre `total`: a
/// sessao seguinte a ultima volta para a primeira.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumeroDeSessao(u8);

impl NumeroDeSessao {
    pub const fn primeira() -> Self {
        Self(1)
    }

    /// Constroi um numero de sessao cru, sem validar contra nenhum total.
    /// Quem chama (redimensionamento, comandos de plano) e responsavel por
    /// garantir que faz sentido no contexto — ver `PlanoDoCiclo`.
    pub const fn de(valor: u8) -> Self {
        Self(valor)
    }

    pub const fn valor(self) -> u8 {
        self.0
    }

    /// `(atual % total) + 1` — so chamado ao entrar em Foco.
    pub const fn proxima(self, total: QuantidadeDeSessoes) -> Self {
        Self((self.0 % total.valor()) + 1)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn total(quantidade: u8) -> QuantidadeDeSessoes {
        QuantidadeDeSessoes::nova(quantidade).expect("teste")
    }

    #[test]
    fn primeira_sessao_e_um() {
        assert_eq!(NumeroDeSessao::primeira().valor(), 1);
    }

    #[test]
    fn sessao_intermediaria_avanca_em_um() {
        assert_eq!(NumeroDeSessao::primeira().proxima(total(4)).valor(), 2);
    }

    #[test]
    fn ultima_sessao_do_ciclo_volta_para_a_primeira() {
        let ultima = NumeroDeSessao::de(4);
        assert_eq!(ultima.proxima(total(4)).valor(), 1);
    }
}
