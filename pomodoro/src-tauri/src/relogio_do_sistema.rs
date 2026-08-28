use chrono::Datelike;
use pomodoro_dominio::{Data, Instante, Relogio};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unica implementacao de `Relogio` que toca `SystemTime`/fuso do SO. O
/// dominio nunca ve isto — recebe a hora por injecao.
pub struct RelogioDoSistema;

impl Relogio for RelogioDoSistema {
    fn agora(&self) -> Instante {
        let decorrido = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Instante::desde_epoca_ms(decorrido.as_millis() as i64)
    }

    /// Fuso local da maquina — e por isto que a virada do dia (PRD §6) e
    /// "meia-noite, horario local", nao meia-noite UTC.
    fn hoje(&self) -> Data {
        let hoje = chrono::Local::now().date_naive();
        Data::de(hoje.year() as u16, hoje.month() as u8, hoje.day() as u8)
    }
}
