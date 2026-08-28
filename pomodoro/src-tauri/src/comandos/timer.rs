use pomodoro_contrato::EstadoDaTela;
use pomodoro_dominio::{CicloEmExecucao, Instante, Relogio};
use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::comandos::historico::HistoricoState;
use crate::comandos::plano::PlanoState;
use crate::relogio_do_sistema::RelogioDoSistema;

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum ErroComando {
    #[error("estado do timer inacessivel: lock envenenado")]
    EstadoInacessivel,
}

type CicloState<'a> = State<'a, Mutex<CicloEmExecucao>>;
type RelogioState<'a> = State<'a, RelogioDoSistema>;

#[tauri::command]
#[specta::specta]
pub fn obter_estado(ciclo: CicloState, relogio: RelogioState) -> Result<EstadoDaTela, ErroComando> {
    let estado = ciclo.lock().map_err(|_| ErroComando::EstadoInacessivel)?;
    Ok(EstadoDaTela::de(&estado, relogio.agora()))
}

#[tauri::command]
#[specta::specta]
pub fn alternar_execucao(
    app: AppHandle,
    ciclo: CicloState,
    relogio: RelogioState,
) -> Result<EstadoDaTela, ErroComando> {
    mutar_e_emitir(&app, ciclo, &relogio, CicloEmExecucao::alternar_execucao)
}

#[tauri::command]
#[specta::specta]
pub fn alterar_iniciar_automaticamente(
    app: AppHandle,
    ciclo: CicloState,
    relogio: RelogioState,
    valor: bool,
) -> Result<EstadoDaTela, ErroComando> {
    let tela = mutar_e_emitir(&app, ciclo, &relogio, |c, _agora| {
        c.com_iniciar_automaticamente(valor)
    })?;
    crate::persistencia::persistir(&app);
    Ok(tela)
}

#[tauri::command]
#[specta::specta]
pub fn reiniciar_etapa(
    app: AppHandle,
    ciclo: CicloState,
    relogio: RelogioState,
) -> Result<EstadoDaTela, ErroComando> {
    mutar_e_emitir(&app, ciclo, &relogio, |c, _agora| c.reiniciar_etapa())
}

/// Alem de avancar a etapa, realinha a duracao ativa com o plano de
/// duracoes: em modo individual, a sessao recem-entrada pode ter uma
/// duracao diferente da que estava valendo antes. Ver
/// [`pomodoro_dominio::PlanoDoCiclo::realinhar_apos_avancar`].
#[tauri::command]
#[specta::specta]
pub fn pular_etapa(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    historico: HistoricoState,
    relogio: RelogioState,
) -> Result<EstadoDaTela, ErroComando> {
    let agora = relogio.agora();
    let hoje = pomodoro_dominio::Relogio::hoje(&*relogio);
    let (ciclo_antes, ciclo_depois, atividade_da_pausa, tela) =
        mutar_para_pular(&ciclo, &plano, agora)?;

    crate::registro_de_foco::registrar_saida_de_foco(
        &app,
        historico,
        hoje,
        &ciclo_antes,
        agora,
        false,
    );
    crate::eventos::emitir(&app, tela);
    crate::persistencia::persistir(&app);
    crate::bloqueio::reagir_a_transicao(
        &app,
        ciclo_antes.etapa(),
        ciclo_depois.etapa(),
        &atividade_da_pausa,
        &tela,
    );
    Ok(tela)
}

/// Isola o lock+mutacao de `pular_etapa` do resto dos efeitos colaterais
/// (historico, evento, persistencia). Os dois guardas sao soltos antes de
/// devolver — quem chama nunca segura `ciclo`/`plano` ao acionar
/// `persistencia::persistir`, que tranca os mesmos mutexes por dentro
/// (deadlock corrigido: guarda de `ciclo` sobrevivia ate o fim da funcao,
/// incluindo a chamada de persistencia).
type MutacaoDePular = (
    CicloEmExecucao,
    CicloEmExecucao,
    pomodoro_dominio::Atividade,
    EstadoDaTela,
);

fn mutar_para_pular(
    ciclo: &Mutex<CicloEmExecucao>,
    plano: &Mutex<pomodoro_dominio::PlanoDoCiclo>,
    agora: Instante,
) -> Result<MutacaoDePular, ErroComando> {
    let mut estado = ciclo.lock().map_err(|_| ErroComando::EstadoInacessivel)?;
    let plano_guarda = plano.lock().map_err(|_| ErroComando::EstadoInacessivel)?;
    let ciclo_antes = *estado;

    let avancado = estado.avancar(agora);
    *estado = plano_guarda.realinhar_apos_avancar(avancado, agora);
    let ciclo_depois = *estado;
    let atividade_da_pausa = plano_guarda.atividade_ativa_para(estado.sessao()).clone();
    drop(plano_guarda);

    let tela = EstadoDaTela::de(&estado, agora);
    Ok((ciclo_antes, ciclo_depois, atividade_da_pausa, tela))
}

fn mutar_e_emitir(
    app: &AppHandle,
    ciclo: CicloState,
    relogio: &RelogioDoSistema,
    transicao: impl FnOnce(CicloEmExecucao, Instante) -> CicloEmExecucao,
) -> Result<EstadoDaTela, ErroComando> {
    let mut estado = ciclo.lock().map_err(|_| ErroComando::EstadoInacessivel)?;
    let agora = relogio.agora();
    *estado = transicao(*estado, agora);
    let tela = EstadoDaTela::de(&estado, agora);
    crate::eventos::emitir(app, tela);
    Ok(tela)
}

#[cfg(test)]
mod testes {
    use super::*;
    use pomodoro_dominio::{Duracao, PlanoDoCiclo, QuantidadeDeSessoes};
    use std::sync::Arc;
    use std::time::Duration;

    fn ciclo_de_teste() -> CicloEmExecucao {
        CicloEmExecucao::novo(
            QuantidadeDeSessoes::nova(4).expect("teste"),
            Duracao::de_minutos(25),
            Duracao::de_minutos(5),
            true,
        )
    }

    fn plano_de_teste() -> PlanoDoCiclo {
        PlanoDoCiclo::novo(
            QuantidadeDeSessoes::nova(4).expect("teste"),
            Duracao::de_minutos(25),
            Duracao::de_minutos(5),
        )
    }

    /// Regressao do deadlock em `pular_etapa`: o guard de `ciclo` sobrevivia
    /// ate a chamada de `persistencia::persistir`, que tranca o mesmo mutex
    /// por dentro — uma thread travando o proprio lock trava pra sempre.
    /// `mutar_para_pular` precisa devolver com os dois guardas ja soltos;
    /// testado travando de fora, de outra thread, logo apos o retorno, com
    /// timeout pra nunca travar a suite caso a regressao volte.
    #[test]
    fn mutar_para_pular_solta_os_locks_antes_de_devolver() {
        let ciclo = Arc::new(Mutex::new(ciclo_de_teste()));
        let plano = Arc::new(Mutex::new(plano_de_teste()));
        let agora = Instante::desde_epoca_ms(0);

        mutar_para_pular(&ciclo, &plano, agora).expect("mutacao nao deve falhar");

        let (ciclo_para_thread, plano_para_thread) = (Arc::clone(&ciclo), Arc::clone(&plano));
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let travou = ciclo_para_thread.lock().is_ok() && plano_para_thread.lock().is_ok();
            let _ = tx.send(travou);
        });

        let travou = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("mutar_para_pular nao soltou os locks (deadlock)");
        assert!(travou);
    }
}
