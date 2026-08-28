use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuantidadeDeSessoesInvalida {
    #[error("quantidade de sessoes invalida: {recebido}, esperado entre 1 e 24")]
    ForaDoIntervalo { recebido: u8 },
}

/// Quantidade de sessoes do ciclo, validada contra os limites fixos do
/// produto (PRD secao 5): 1 a 24.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuantidadeDeSessoes(u8);

impl QuantidadeDeSessoes {
    pub const MINIMA: u8 = 1;
    pub const MAXIMA: u8 = 24;

    pub fn nova(quantidade: u8) -> Result<Self, QuantidadeDeSessoesInvalida> {
        if !(Self::MINIMA..=Self::MAXIMA).contains(&quantidade) {
            return Err(QuantidadeDeSessoesInvalida::ForaDoIntervalo {
                recebido: quantidade,
            });
        }
        Ok(Self(quantidade))
    }

    pub const fn valor(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn zero_e_invalido() {
        let erro = QuantidadeDeSessoes::nova(0).expect_err("teste");
        assert_eq!(
            erro,
            QuantidadeDeSessoesInvalida::ForaDoIntervalo { recebido: 0 }
        );
        assert_eq!(
            erro.to_string(),
            "quantidade de sessoes invalida: 0, esperado entre 1 e 24"
        );
    }

    #[test]
    fn vinte_e_cinco_e_invalido() {
        let erro = QuantidadeDeSessoes::nova(25).expect_err("teste");
        assert_eq!(
            erro,
            QuantidadeDeSessoesInvalida::ForaDoIntervalo { recebido: 25 }
        );
    }

    #[test]
    fn borda_inferior_um_e_valida() {
        assert_eq!(QuantidadeDeSessoes::nova(1).expect("teste").valor(), 1);
    }

    #[test]
    fn borda_superior_vinte_e_quatro_e_valida() {
        assert_eq!(QuantidadeDeSessoes::nova(24).expect("teste").valor(), 24);
    }
}
