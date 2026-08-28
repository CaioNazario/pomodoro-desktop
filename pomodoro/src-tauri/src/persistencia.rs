use pomodoro_config::{Armazenamento, Configuracao};
use pomodoro_dominio::{CicloEmExecucao, HistoricoDiario, PlanoDoCiclo};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const NOME_DO_ARQUIVO: &str = "config.toml";

/// `None` so se o Tauri nao conseguir resolver o diretorio de config do SO
/// — nesse caso a persistencia e pulada por inteiro, nunca panica.
fn armazenamento(app: &AppHandle) -> Option<Armazenamento> {
    let diretorio = app.path().app_config_dir().ok()?;
    Some(Armazenamento::em(diretorio.join(NOME_DO_ARQUIVO)))
}

/// Carrega do disco ou cai no default — nunca falha a inicializacao.
pub fn carregar(app: &AppHandle, construir_padrao: impl FnOnce() -> Configuracao) -> Configuracao {
    match armazenamento(app) {
        Some(armazenamento) => armazenamento.carregar(construir_padrao),
        None => construir_padrao(),
    }
}

/// Melhor esforco: chamado apos toda mutacao de plano/historico/autostart.
/// Falha de escrita e so registrada — nunca interrompe o command que
/// disparou a persistencia nem propaga como erro pro frontend.
pub fn persistir(app: &AppHandle) {
    let (Some(armazenamento), Some(config)) = (armazenamento(app), configuracao_atual(app)) else {
        return;
    };
    if let Err(erro) = armazenamento.salvar(&config) {
        eprintln!("pomodoro-config: falha ao persistir: {erro}");
    }
}

fn configuracao_atual(app: &AppHandle) -> Option<Configuracao> {
    let ciclo_state = app.try_state::<Mutex<CicloEmExecucao>>()?;
    let plano_state = app.try_state::<Mutex<PlanoDoCiclo>>()?;
    let historico_state = app.try_state::<Mutex<HistoricoDiario>>()?;
    let ciclo = ciclo_state.lock().ok()?;
    let plano = plano_state.lock().ok()?;
    let historico = historico_state.lock().ok()?;
    let posicao_do_widget = app
        .try_state::<crate::widget::EstadoDoWidget>()
        .and_then(|estado| estado.posicao_atual());
    Some(Configuracao {
        plano: plano.clone(),
        iniciar_automaticamente: ciclo.iniciar_automaticamente(),
        historico: historico.clone(),
        posicao_do_widget,
    })
}
