use crate::Duracao;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DuracaoDeEtapaInvalida {
    #[error("duracao de foco invalida: {recebido}min, esperado entre 5 e 180 em passos de 5")]
    Foco { recebido: u16 },
    #[error("duracao de pausa invalida: {recebido}min, esperado entre 1 e 60")]
    Pausa { recebido: u16 },
}

const FOCO_MINIMA_MIN: u16 = 5;
const FOCO_MAXIMA_MIN: u16 = 180;
const FOCO_PASSO_MIN: u16 = 5;
const PAUSA_MINIMA_MIN: u16 = 1;
const PAUSA_MAXIMA_MIN: u16 = 60;

impl Duracao {
    /// Limites fixos do PRD secao 5: 5..=180min, em passos de 5. O passo e
    /// validado aqui, no dominio — validacao no frontend e so UX.
    pub fn de_minutos_de_foco(minutos: u16) -> Result<Self, DuracaoDeEtapaInvalida> {
        let dentro_do_intervalo = (FOCO_MINIMA_MIN..=FOCO_MAXIMA_MIN).contains(&minutos);
        let no_passo = minutos.is_multiple_of(FOCO_PASSO_MIN);
        if !dentro_do_intervalo || !no_passo {
            return Err(DuracaoDeEtapaInvalida::Foco { recebido: minutos });
        }
        Ok(Self::de_minutos(minutos))
    }

    /// Limites fixos do PRD secao 5: 1..=60min, sem passo (todo inteiro
    /// nesse intervalo e valido).
    pub fn de_minutos_de_pausa(minutos: u16) -> Result<Self, DuracaoDeEtapaInvalida> {
        let dentro_do_intervalo = (PAUSA_MINIMA_MIN..=PAUSA_MAXIMA_MIN).contains(&minutos);
        if !dentro_do_intervalo {
            return Err(DuracaoDeEtapaInvalida::Pausa { recebido: minutos });
        }
        Ok(Self::de_minutos(minutos))
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn foco_borda_inferior_e_superior_sao_validas() {
        assert!(Duracao::de_minutos_de_foco(5).is_ok());
        assert!(Duracao::de_minutos_de_foco(180).is_ok());
    }

    #[test]
    fn foco_fora_do_intervalo_e_invalido() {
        let erro = Duracao::de_minutos_de_foco(0).expect_err("teste");
        assert_eq!(erro, DuracaoDeEtapaInvalida::Foco { recebido: 0 });
        assert!(Duracao::de_minutos_de_foco(185).is_err());
    }

    #[test]
    fn foco_fora_do_passo_de_cinco_e_invalido() {
        let erro = Duracao::de_minutos_de_foco(7).expect_err("teste");
        assert_eq!(erro, DuracaoDeEtapaInvalida::Foco { recebido: 7 });
    }

    #[test]
    fn pausa_borda_inferior_e_superior_sao_validas() {
        assert!(Duracao::de_minutos_de_pausa(1).is_ok());
        assert!(Duracao::de_minutos_de_pausa(60).is_ok());
    }

    #[test]
    fn pausa_fora_do_intervalo_e_invalida() {
        let erro = Duracao::de_minutos_de_pausa(0).expect_err("teste");
        assert_eq!(erro, DuracaoDeEtapaInvalida::Pausa { recebido: 0 });
        assert!(Duracao::de_minutos_de_pausa(65).is_err());
    }
}
