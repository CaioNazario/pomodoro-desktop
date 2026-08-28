/// Fonte das duracoes de foco/pausa usadas pelo ciclo: globais (um par para
/// todas as sessoes) ou individuais (um par por sessao). Os dois conjuntos
/// coexistem sempre — ver [`crate::PlanoDoCiclo`]; isto so seleciona qual
/// esta ativo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoDeDuracao {
    Global,
    Individual,
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn variantes_sao_distintas() {
        assert_ne!(ModoDeDuracao::Global, ModoDeDuracao::Individual);
    }
}
