use pomodoro_contrato::{EstadoDaTela, EstadoDoBloqueio, EstadoDoHistorico, EstadoDoPlano};
use tauri::AppHandle;
use tauri_specta::Event;

/// Emitido so em transicao de estado — o Rust nunca manda tick a 1Hz.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct EstadoMudou(pub EstadoDaTela);

/// Emitido quando o plano de duracoes muda: redimensionamento, troca de
/// modo ou alteracao de duracao (global ou individual).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct PlanoMudou(pub EstadoDoPlano);

/// Emitido quando os contadores do dia mudam: sessao concluida, foco
/// pulado pela metade ou pausa interrompida.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct HistoricoMudou(pub EstadoDoHistorico);

/// Emitido so ao entrar/sair do bloqueio (PRD §7) — nunca a 1Hz.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct BloqueioMudou(pub EstadoDoBloqueio);

/// Emitido so quando a janela principal realmente é trazida pra frente
/// (PRD §8) — nunca inferido no frontend a partir da transicao de etapa,
/// porque isso teria que reimplementar as duas condicoes que o Rust ja
/// decide (fim de Foco automatico, nunca skip manual; so com URL
/// aplicavel).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct TomadaDeFocoOcorreu;

pub fn emitir(app: &AppHandle, tela: EstadoDaTela) {
    let _ = EstadoMudou(tela).emit(app);
}

pub fn emitir_plano(app: &AppHandle, plano: EstadoDoPlano) {
    let _ = PlanoMudou(plano).emit(app);
}

pub fn emitir_historico(app: &AppHandle, historico: EstadoDoHistorico) {
    let _ = HistoricoMudou(historico).emit(app);
}

pub fn emitir_bloqueio(app: &AppHandle, bloqueio: EstadoDoBloqueio) {
    let _ = BloqueioMudou(bloqueio).emit(app);
}

pub fn emitir_tomada_de_foco(app: &AppHandle) {
    let _ = TomadaDeFocoOcorreu.emit(app);
}
