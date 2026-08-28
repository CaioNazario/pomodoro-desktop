use std::sync::atomic::{AtomicU64, Ordering};

/// Coordena o debounce de escrita em disco durante um arrasto continuo do
/// widget: cada `Moved` chama `registrar_movimento`, que devolve um numero
/// de geracao. Se, apos o atraso de persistencia, essa ainda for a geracao
/// mais recente, nenhum `Moved` novo chegou nesse meio tempo e e seguro
/// gravar — sem isso, um arrasto de mouse dispararia dezenas de escritas
/// atomicas em disco por segundo.
pub(super) struct RastreadorDeArrasto {
    geracao: AtomicU64,
}

impl RastreadorDeArrasto {
    pub(super) fn novo() -> Self {
        Self {
            geracao: AtomicU64::new(0),
        }
    }

    pub(super) fn registrar_movimento(&self) -> u64 {
        self.geracao.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub(super) fn ainda_e_a_ultima(&self, geracao: u64) -> bool {
        self.geracao.load(Ordering::SeqCst) == geracao
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn primeiro_movimento_e_a_ultima_geracao() {
        let rastreador = RastreadorDeArrasto::novo();
        let geracao = rastreador.registrar_movimento();
        assert!(rastreador.ainda_e_a_ultima(geracao));
    }

    #[test]
    fn movimento_seguinte_invalida_a_geracao_anterior() {
        let rastreador = RastreadorDeArrasto::novo();
        let primeira = rastreador.registrar_movimento();
        rastreador.registrar_movimento();
        assert!(!rastreador.ainda_e_a_ultima(primeira));
    }

    #[test]
    fn geracao_mais_recente_permanece_valida() {
        let rastreador = RastreadorDeArrasto::novo();
        rastreador.registrar_movimento();
        let segunda = rastreador.registrar_movimento();
        assert!(rastreador.ainda_e_a_ultima(segunda));
    }
}
