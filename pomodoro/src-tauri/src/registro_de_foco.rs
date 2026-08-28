use pomodoro_contrato::EstadoDoHistorico;
use pomodoro_dominio::{CicloEmExecucao, Data, Etapa, HistoricoDiario, Instante, Relogio};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::comandos::historico::HistoricoState;
use crate::relogio_do_sistema::RelogioDoSistema;

/// Registra, no historico do dia, o Foco que esta sendo deixado — chamado
/// tanto por `pular_etapa` (nao conta como sessao concluida, so soma o
/// tempo real decorrido) quanto pelo vencimento natural do prazo (conta
/// como sessao concluida). Nao faz nada se a etapa que esta sendo deixada
/// nao for Foco. Nunca panica: lock envenenado so faz o registro ser
/// pulado.
pub fn registrar_saida_de_foco(
    app: &AppHandle,
    historico: HistoricoState,
    hoje: Data,
    ciclo_antes: &CicloEmExecucao,
    agora: Instante,
    concluiu_naturalmente: bool,
) {
    if ciclo_antes.etapa() != Etapa::Foco {
        return;
    }
    let Ok(mut guarda) = historico.lock() else {
        return;
    };
    let decorrido = ciclo_antes
        .duracao_da_etapa_atual()
        .menos(ciclo_antes.restante_em(agora));
    let atualizado = if concluiu_naturalmente {
        guarda.clone().registrar_sessao_concluida(hoje, decorrido)
    } else {
        guarda.clone().registrar_foco_parcial(hoje, decorrido)
    };
    let dia_atual = atualizado.dia_atual(hoje);
    *guarda = atualizado;
    drop(guarda);
    crate::eventos::emitir_historico(app, EstadoDoHistorico::de(dia_atual));
}

/// Fuga detectada sob bloqueio (PRD §7.4): resolve o proprio state, porque
/// quem chama e o handler de `WindowEvent::Focused`, sem `State<T>` tipado
/// como parametro de command. Nunca panica: qualquer state ausente ou lock
/// envenenado so faz o registro ser pulado.
pub fn registrar_pausa_interrompida(app: &AppHandle) {
    let (Some(historico), Some(relogio)) = (
        app.try_state::<Mutex<HistoricoDiario>>(),
        app.try_state::<RelogioDoSistema>(),
    ) else {
        return;
    };
    let hoje = relogio.hoje();
    let Ok(mut guarda) = historico.lock() else {
        return;
    };
    let atualizado = guarda.clone().registrar_pausa_interrompida(hoje);
    let dia_atual = atualizado.dia_atual(hoje);
    *guarda = atualizado;
    drop(guarda);
    crate::eventos::emitir_historico(app, EstadoDoHistorico::de(dia_atual));
}
