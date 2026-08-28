/// Data de calendario (ano-mes-dia), sem fuso. O dominio nao resolve fuso
/// horario nenhum — quem decide o que e "hoje" e o [`crate::Relogio`]
/// injetado, via `hoje()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Data {
    ano: u16,
    mes: u8,
    dia: u8,
}

impl Data {
    pub const fn de(ano: u16, mes: u8, dia: u8) -> Self {
        Self { ano, mes, dia }
    }

    pub const fn ano(self) -> u16 {
        self.ano
    }

    pub const fn mes(self) -> u8 {
        self.mes
    }

    pub const fn dia(self) -> u8 {
        self.dia
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn datas_iguais_sao_iguais_e_comparaveis_por_ordem_cronologica() {
        assert_eq!(Data::de(2026, 8, 23), Data::de(2026, 8, 23));
        assert!(Data::de(2026, 8, 23) < Data::de(2026, 8, 24));
        assert!(Data::de(2026, 8, 31) < Data::de(2026, 9, 1));
    }
}
