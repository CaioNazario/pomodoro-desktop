//! Relogio de teste, compartilhado por todos os modulos do dominio.
//! Nenhum teste de pomodoro espera 25 minutos de verdade.

use crate::{Data, Duracao, Instante, Relogio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

pub(crate) struct RelogioFake {
    epoca_ms: AtomicI64,
    hoje: Mutex<Data>,
}

impl RelogioFake {
    pub(crate) fn parado_em(instante: Instante) -> Self {
        Self {
            epoca_ms: AtomicI64::new(instante.epoca_ms()),
            hoje: Mutex::new(Data::de(2026, 1, 1)),
        }
    }

    pub(crate) fn avancar(&self, quanto: Duracao) {
        self.epoca_ms.fetch_add(quanto.em_ms(), Ordering::SeqCst);
    }

    /// Simula a virada do dia — desacoplado de `epoca_ms` de proposito,
    /// porque quem resolve fuso na vida real e a implementacao de sistema,
    /// nao o relogio de teste.
    pub(crate) fn mudar_dia(&self, novo: Data) {
        *self.hoje.lock().expect("lock do dia fake") = novo;
    }
}

impl Relogio for RelogioFake {
    fn agora(&self) -> Instante {
        Instante::desde_epoca_ms(self.epoca_ms.load(Ordering::SeqCst))
    }

    fn hoje(&self) -> Data {
        *self.hoje.lock().expect("lock do dia fake")
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn relogio_fake_devolve_o_instante_que_mandarem() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(42));
        assert_eq!(relogio.agora(), Instante::desde_epoca_ms(42));
    }

    #[test]
    fn avancar_move_o_relogio_sem_esperar_tempo_real() {
        let relogio = RelogioFake::parado_em(Instante::desde_epoca_ms(0));
        relogio.avancar(Duracao::de_minutos(25));
        assert_eq!(relogio.agora().epoca_ms(), 1_500_000);
    }
}
