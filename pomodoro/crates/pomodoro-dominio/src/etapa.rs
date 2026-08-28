/// A menor unidade de tempo do ciclo: `Foco` ou `Pausa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Etapa {
    Foco,
    Pausa,
}

impl Etapa {
    pub const fn alternar(self) -> Self {
        match self {
            Etapa::Foco => Etapa::Pausa,
            Etapa::Pausa => Etapa::Foco,
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn foco_alterna_para_pausa() {
        assert_eq!(Etapa::Foco.alternar(), Etapa::Pausa);
    }

    #[test]
    fn pausa_alterna_para_foco() {
        assert_eq!(Etapa::Pausa.alternar(), Etapa::Foco);
    }
}
