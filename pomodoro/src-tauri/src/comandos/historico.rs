use pomodoro_contrato::EstadoDoHistorico;
use pomodoro_dominio::HistoricoDiario;
use std::sync::Mutex;
use tauri::State;

use crate::comandos::timer::ErroComando;
use crate::relogio_do_sistema::RelogioDoSistema;

pub type HistoricoState<'a> = State<'a, Mutex<HistoricoDiario>>;

#[tauri::command]
#[specta::specta]
pub fn obter_contadores_do_dia(
    historico: HistoricoState,
    relogio: State<RelogioDoSistema>,
) -> Result<EstadoDoHistorico, ErroComando> {
    let historico = historico
        .lock()
        .map_err(|_| ErroComando::EstadoInacessivel)?;
    let hoje = pomodoro_dominio::Relogio::hoje(&*relogio);
    Ok(EstadoDoHistorico::de(historico.dia_atual(hoje)))
}
