use pomodoro_contrato::SomDeAlarme;
use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::comandos::timer::ErroComando;

pub type SomDeAlarmeState<'a> = State<'a, Mutex<pomodoro_dominio::SomDeAlarme>>;

/// Preferencia de som de alarme (PRD §7): qual dos 3 sons tocar nas
/// transicoes de etapa de sessoes sem URL aplicavel (ver `crate::alarme`).
/// Nao afeta ciclo nem plano — so a config persistida.
#[tauri::command]
#[specta::specta]
pub fn obter_som_de_alarme(som: SomDeAlarmeState) -> Result<SomDeAlarme, ErroComando> {
    let som = som.lock().map_err(|_| ErroComando::EstadoInacessivel)?;
    Ok((*som).into())
}

#[tauri::command]
#[specta::specta]
pub fn alterar_som_de_alarme(
    app: AppHandle,
    som: SomDeAlarmeState,
    novo: SomDeAlarme,
) -> Result<(), ErroComando> {
    {
        let mut guarda = som.lock().map_err(|_| ErroComando::EstadoInacessivel)?;
        *guarda = novo.into();
    }
    crate::eventos::emitir_som_de_alarme(&app, novo);
    crate::persistencia::persistir(&app);
    Ok(())
}
