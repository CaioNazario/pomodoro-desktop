use pomodoro_config::{Armazenamento, Configuracao};
use pomodoro_dominio::{CicloEmExecucao, HistoricoDiario, PlanoDoCiclo};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const NOME_DO_ARQUIVO: &str = "config.toml";
const NOME_DO_BANCO: &str = "historico.db";

/// `None` so se o Tauri nao conseguir resolver o diretorio de config do SO
/// — nesse caso a persistencia e pulada por inteiro, nunca panica.
fn armazenamento(app: &AppHandle) -> Option<Armazenamento> {
    let diretorio = app.path().app_config_dir().ok()?;
    Some(Armazenamento::em(diretorio.join(NOME_DO_ARQUIVO)))
}

fn caminho_do_banco(app: &AppHandle) -> Option<PathBuf> {
    let diretorio = app.path().app_config_dir().ok()?;
    Some(diretorio.join(NOME_DO_BANCO))
}

/// Carrega a configuracao do TOML e o historico do SQLite — migra o
/// historico embutido num `config.toml` de antes do SQLite na primeira vez
/// que `historico.db` ainda nao existir. Nunca falha a inicializacao.
pub fn carregar(
    app: &AppHandle,
    construir_padrao: impl FnOnce() -> Configuracao,
) -> (Configuracao, HistoricoDiario) {
    let config = match armazenamento(app) {
        Some(armazenamento) => armazenamento.carregar(construir_padrao),
        None => construir_padrao(),
    };
    let historico = carregar_historico(app);
    (config, historico)
}

fn carregar_historico(app: &AppHandle) -> HistoricoDiario {
    let Some(caminho) = caminho_do_banco(app) else {
        return HistoricoDiario::vazio();
    };
    let banco_e_novo = !caminho.exists();
    match pomodoro_historico::Armazenamento::abrir(caminho) {
        Ok(mut banco) => {
            if banco_e_novo {
                let legado = armazenamento(app)
                    .map(|a| a.historico_legado())
                    .unwrap_or_default();
                if let Err(erro) = banco.migrar_historico_legado(legado) {
                    eprintln!("pomodoro-historico: falha ao migrar historico legado: {erro}");
                }
            }
            banco.carregar()
        }
        Err(erro) => {
            eprintln!("pomodoro-historico: falha ao abrir o banco: {erro}");
            HistoricoDiario::vazio()
        }
    }
}

/// Melhor esforco: chamado apos toda mutacao de plano/historico/autostart.
/// Falha de escrita e so registrada — nunca interrompe o command que
/// disparou a persistencia nem propaga como erro pro frontend.
pub fn persistir(app: &AppHandle) {
    if let (Some(armazenamento), Some(config)) = (armazenamento(app), configuracao_atual(app)) {
        if let Err(erro) = armazenamento.salvar(&config) {
            eprintln!("pomodoro-config: falha ao persistir: {erro}");
        }
    }
    persistir_historico(app);
}

fn persistir_historico(app: &AppHandle) {
    let (Some(caminho), Some(historico_state)) = (
        caminho_do_banco(app),
        app.try_state::<Mutex<HistoricoDiario>>(),
    ) else {
        return;
    };
    let Ok(historico) = historico_state.lock() else {
        return;
    };
    match pomodoro_historico::Armazenamento::abrir(caminho) {
        Ok(mut banco) => {
            if let Err(erro) = banco.salvar(&historico) {
                eprintln!("pomodoro-historico: falha ao persistir: {erro}");
            }
        }
        Err(erro) => eprintln!("pomodoro-historico: falha ao abrir o banco: {erro}"),
    }
}

fn configuracao_atual(app: &AppHandle) -> Option<Configuracao> {
    let ciclo_state = app.try_state::<Mutex<CicloEmExecucao>>()?;
    let plano_state = app.try_state::<Mutex<PlanoDoCiclo>>()?;
    let ciclo = ciclo_state.lock().ok()?;
    let plano = plano_state.lock().ok()?;
    let posicao_do_widget = app
        .try_state::<crate::widget::EstadoDoWidget>()
        .and_then(|estado| estado.posicao_atual());
    Some(Configuracao {
        plano: plano.clone(),
        iniciar_automaticamente: ciclo.iniciar_automaticamente(),
        posicao_do_widget,
    })
}
