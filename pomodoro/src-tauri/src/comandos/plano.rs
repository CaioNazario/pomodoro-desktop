use pomodoro_contrato::EstadoDoPlano;
use pomodoro_dominio::{
    Atividade, CicloEmExecucao, Duracao, DuracaoDeEtapaInvalida, Instante, NumeroDeSessao,
    PlanoDoCiclo, QuantidadeDeSessoes, QuantidadeDeSessoesInvalida, Relogio, SessaoForaDoPlano,
    UrlDeAtividade, UrlDeAtividadeInvalida,
};
use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::relogio_do_sistema::RelogioDoSistema;

type CicloState<'a> = State<'a, Mutex<CicloEmExecucao>>;
pub type PlanoState<'a> = State<'a, Mutex<PlanoDoCiclo>>;
type RelogioState<'a> = State<'a, RelogioDoSistema>;

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum ErroComandoDePlano {
    #[error("estado do timer inacessivel: lock envenenado")]
    EstadoInacessivel,
    #[error("quantidade de sessoes invalida: {recebido}, esperado entre 1 e 24")]
    QuantidadeDeSessoesInvalida { recebido: u8 },
    #[error("duracao de foco invalida: {recebido}min, esperado entre 5 e 180 em passos de 5")]
    DuracaoDeFocoInvalida { recebido: u16 },
    #[error("duracao de pausa invalida: {recebido}min, esperado entre 1 e 60")]
    DuracaoDePausaInvalida { recebido: u16 },
    #[error("sessao fora do plano: {recebido}, esperado entre 1 e {total}")]
    SessaoForaDoPlano { recebido: u8, total: u8 },
    #[error("url de atividade invalida: '{recebido}' nao e uma url")]
    UrlDeAtividadeNaoEUmaUrl { recebido: String },
    #[error("esquema de url nao permitido: '{esquema}', esperado http ou https")]
    EsquemaDeUrlNaoPermitido { esquema: String },
}

impl From<QuantidadeDeSessoesInvalida> for ErroComandoDePlano {
    fn from(erro: QuantidadeDeSessoesInvalida) -> Self {
        let QuantidadeDeSessoesInvalida::ForaDoIntervalo { recebido } = erro;
        Self::QuantidadeDeSessoesInvalida { recebido }
    }
}

impl From<DuracaoDeEtapaInvalida> for ErroComandoDePlano {
    fn from(erro: DuracaoDeEtapaInvalida) -> Self {
        match erro {
            DuracaoDeEtapaInvalida::Foco { recebido } => Self::DuracaoDeFocoInvalida { recebido },
            DuracaoDeEtapaInvalida::Pausa { recebido } => Self::DuracaoDePausaInvalida { recebido },
        }
    }
}

impl From<SessaoForaDoPlano> for ErroComandoDePlano {
    fn from(erro: SessaoForaDoPlano) -> Self {
        let SessaoForaDoPlano::ForaDoIntervalo { recebido, total } = erro;
        Self::SessaoForaDoPlano { recebido, total }
    }
}

impl From<UrlDeAtividadeInvalida> for ErroComandoDePlano {
    fn from(erro: UrlDeAtividadeInvalida) -> Self {
        match erro {
            UrlDeAtividadeInvalida::NaoEUmaUrl { recebido } => {
                Self::UrlDeAtividadeNaoEUmaUrl { recebido }
            }
            UrlDeAtividadeInvalida::EsquemaNaoPermitido { esquema } => {
                Self::EsquemaDeUrlNaoPermitido { esquema }
            }
        }
    }
}

fn atividade_de(
    nome: Option<String>,
    url: Option<String>,
) -> Result<Atividade, ErroComandoDePlano> {
    let url = url.map(|bruta| UrlDeAtividade::nova(&bruta)).transpose()?;
    Ok(Atividade::nova(nome, url))
}

#[tauri::command]
#[specta::specta]
pub fn obter_plano(plano: PlanoState) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let estado = plano
        .lock()
        .map_err(|_| ErroComandoDePlano::EstadoInacessivel)?;
    Ok(EstadoDoPlano::de(&estado))
}

#[tauri::command]
#[specta::specta]
pub fn redimensionar(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    nova_quantidade: u8,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let quantidade = QuantidadeDeSessoes::nova(nova_quantidade)?;
    mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, _agora| {
        p.redimensionar(quantidade, c)
    })
}

#[tauri::command]
#[specta::specta]
pub fn trocar_modo(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    novo_modo: pomodoro_contrato::ModoDeDuracao,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let novo_modo = novo_modo.into();
    mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, agora| {
        p.trocar_modo(novo_modo, c, agora)
    })
}

#[tauri::command]
#[specta::specta]
pub fn alterar_duracao_global_foco(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    minutos: u16,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let nova = Duracao::de_minutos_de_foco(minutos)?;
    mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, agora| {
        p.alterar_duracao_global_foco(nova, c, agora)
    })
}

#[tauri::command]
#[specta::specta]
pub fn alterar_duracao_global_pausa(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    minutos: u16,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let nova = Duracao::de_minutos_de_pausa(minutos)?;
    mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, agora| {
        p.alterar_duracao_global_pausa(nova, c, agora)
    })
}

#[tauri::command]
#[specta::specta]
pub fn alterar_duracao_individual_foco(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    numero_sessao: u8,
    minutos: u16,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let nova = Duracao::de_minutos_de_foco(minutos)?;
    let sessao = NumeroDeSessao::de(numero_sessao);
    tentar_mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, agora| {
        p.alterar_duracao_individual_foco(sessao, nova, c, agora)
    })
}

#[tauri::command]
#[specta::specta]
pub fn alterar_duracao_individual_pausa(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    numero_sessao: u8,
    minutos: u16,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let nova = Duracao::de_minutos_de_pausa(minutos)?;
    let sessao = NumeroDeSessao::de(numero_sessao);
    tentar_mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, agora| {
        p.alterar_duracao_individual_pausa(sessao, nova, c, agora)
    })
}

#[tauri::command]
#[specta::specta]
pub fn alterar_atividade_global(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    nome: Option<String>,
    url: Option<String>,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let atividade = atividade_de(nome, url)?;
    mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, _agora| {
        (p.alterar_atividade_global(atividade), c)
    })
}

#[tauri::command]
#[specta::specta]
pub fn alterar_atividade_individual(
    app: AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: RelogioState,
    numero_sessao: u8,
    nome: Option<String>,
    url: Option<String>,
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    let atividade = atividade_de(nome, url)?;
    let sessao = NumeroDeSessao::de(numero_sessao);
    tentar_mutar_e_emitir(&app, ciclo, plano, &relogio, |p, c, _agora| {
        p.alterar_atividade_individual(sessao, atividade)
            .map(|p| (p, c))
    })
}

fn mutar_e_emitir(
    app: &AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: &RelogioDoSistema,
    transicao: impl FnOnce(PlanoDoCiclo, CicloEmExecucao, Instante) -> (PlanoDoCiclo, CicloEmExecucao),
) -> Result<EstadoDoPlano, ErroComandoDePlano> {
    tentar_mutar_e_emitir(app, ciclo, plano, relogio, |p, c, agora| {
        Ok::<_, SessaoForaDoPlano>(transicao(p, c, agora))
    })
}

fn tentar_mutar_e_emitir<E>(
    app: &AppHandle,
    ciclo: CicloState,
    plano: PlanoState,
    relogio: &RelogioDoSistema,
    transicao: impl FnOnce(
        PlanoDoCiclo,
        CicloEmExecucao,
        Instante,
    ) -> Result<(PlanoDoCiclo, CicloEmExecucao), E>,
) -> Result<EstadoDoPlano, ErroComandoDePlano>
where
    ErroComandoDePlano: From<E>,
{
    let mut estado_ciclo = ciclo
        .lock()
        .map_err(|_| ErroComandoDePlano::EstadoInacessivel)?;
    let mut estado_plano = plano
        .lock()
        .map_err(|_| ErroComandoDePlano::EstadoInacessivel)?;
    let agora = relogio.agora();
    let (novo_plano, novo_ciclo) = transicao(estado_plano.clone(), *estado_ciclo, agora)?;
    *estado_plano = novo_plano;
    *estado_ciclo = novo_ciclo;
    crate::eventos::emitir(
        app,
        pomodoro_contrato::EstadoDaTela::de(&estado_ciclo, agora),
    );
    let tela_do_plano = EstadoDoPlano::de(&estado_plano);
    crate::eventos::emitir_plano(app, tela_do_plano.clone());
    drop(estado_ciclo);
    drop(estado_plano);
    crate::persistencia::persistir(app);
    Ok(tela_do_plano)
}
