use pomodoro_contrato::EstadoDaTela;
use pomodoro_dominio::{CicloEmExecucao, HistoricoDiario, Instante, PlanoDoCiclo, Relogio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

use crate::relogio_do_sistema::RelogioDoSistema;

const INTERVALO_DE_VERIFICACAO: Duration = Duration::from_millis(300);

/// Detecta o vencimento do prazo e aplica `avancar` uma unica vez — o Rust
/// nao emite tick, entao ninguem mais chama isto. Sem suprimir instantes
/// intermediarios: uma suspensao de horas ainda produz uma so transicao,
/// porque `avancar` recalcula o prazo a partir de `agora`.
pub fn iniciar(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(INTERVALO_DE_VERIFICACAO);
        verificar(&app);
    });
}

fn verificar(app: &AppHandle) {
    let Some((ciclo_state, plano_state, historico_state, relogio_state)) = estados_gerenciados(app)
    else {
        return;
    };
    let Ok(mut estado) = ciclo_state.lock() else {
        return;
    };
    let Ok(plano) = plano_state.lock() else {
        return;
    };

    let agora = relogio_state.agora();
    if !estado.venceu_em(agora) {
        return;
    }

    let ciclo_antes = *estado;
    let hoje = relogio_state.hoje();
    let tela_depois = avancar_e_emitir(app, &mut estado, &plano, agora);
    let etapa_depois = estado.etapa();
    let atividade_da_pausa = plano.atividade_ativa_para(estado.sessao()).clone();
    drop(estado);
    drop(plano);

    crate::registro_de_foco::registrar_saida_de_foco(
        app,
        historico_state,
        hoje,
        &ciclo_antes,
        agora,
        true,
    );
    crate::persistencia::persistir(app);
    crate::bloqueio::reagir_a_transicao(
        app,
        ciclo_antes.etapa(),
        etapa_depois,
        &atividade_da_pausa,
        &tela_depois,
    );
    if crate::tomada_de_foco::deve_tomar_foco(
        ciclo_antes.etapa(),
        etapa_depois,
        &atividade_da_pausa,
    ) {
        tomar_foco_da_janela_principal(app);
    }
}

/// Best-effort (PRD §8): a janela principal sobe pra frente, mas uma falha
/// aqui nunca deve derrubar o verificador de vencimento. O evento e emitido
/// mesmo se o `set_focus` falhar — o overlay reflete a intencao de tomar
/// foco, nao a garantia de que o WM obedeceu.
fn tomar_foco_da_janela_principal(app: &AppHandle) {
    crate::eventos::emitir_tomada_de_foco(app);
    let Some(janela) = app.get_webview_window("main") else {
        return;
    };
    if let Err(erro) = janela.set_focus() {
        eprintln!("tomada-de-foco: falha ao focar a janela principal: {erro}");
    }
}

type EstadosGerenciados<'a> = (
    State<'a, Mutex<CicloEmExecucao>>,
    State<'a, Mutex<PlanoDoCiclo>>,
    State<'a, Mutex<HistoricoDiario>>,
    State<'a, RelogioDoSistema>,
);

fn estados_gerenciados(app: &AppHandle) -> Option<EstadosGerenciados<'_>> {
    Some((
        app.try_state()?,
        app.try_state()?,
        app.try_state()?,
        app.try_state()?,
    ))
}

fn avancar_e_emitir(
    app: &AppHandle,
    estado: &mut CicloEmExecucao,
    plano: &PlanoDoCiclo,
    agora: Instante,
) -> EstadoDaTela {
    let avancado = estado.avancar(agora);
    *estado = plano.realinhar_apos_avancar(avancado, agora);
    let tela = EstadoDaTela::de(estado, agora);
    crate::eventos::emitir(app, tela);
    tela
}
