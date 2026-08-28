#![forbid(unsafe_code)]

//! Tipos que atravessam a fronteira Rust<->TypeScript, gerados como
//! `src/bindings.ts` pelo `tauri-specta` no `setup()` do `src-tauri`.

use pomodoro_dominio::{CicloEmExecucao, ContadoresDoDia, Instante, PlanoDoCiclo};
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Etapa {
    Foco,
    Pausa,
}

impl From<pomodoro_dominio::Etapa> for Etapa {
    fn from(etapa: pomodoro_dominio::Etapa) -> Self {
        match etapa {
            pomodoro_dominio::Etapa::Foco => Etapa::Foco,
            pomodoro_dominio::Etapa::Pausa => Etapa::Pausa,
        }
    }
}

/// Retrato do ciclo num instante. O prazo e absoluto: o React deriva
/// `mm:ss` localmente a partir dele via `requestAnimationFrame`, sem
/// receber tick do Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDaTela {
    pub etapa: Etapa,
    pub numero_sessao: u8,
    pub total_sessoes: u8,
    /// `i64` forcado a `number` no TS: ms desde a epoca cabe com folga em
    /// `Number.MAX_SAFE_INTEGER` por seculos, e o dominio ja garante >= 0.
    #[specta(type = Option<Number>)]
    pub prazo_epoca_ms: Option<i64>,
    #[specta(type = Number)]
    pub restante_ms: i64,
    pub rodando: bool,
    #[specta(type = Number)]
    pub duracao_total_ms: i64,
    pub iniciar_automaticamente: bool,
}

impl EstadoDaTela {
    pub fn de(ciclo: &CicloEmExecucao, agora: Instante) -> Self {
        Self {
            etapa: ciclo.etapa().into(),
            numero_sessao: ciclo.sessao().valor(),
            total_sessoes: ciclo.total_sessoes(),
            prazo_epoca_ms: ciclo.prazo().map(Instante::epoca_ms),
            restante_ms: ciclo.restante_em(agora).em_ms(),
            rodando: ciclo.rodando(),
            duracao_total_ms: ciclo.duracao_da_etapa_atual().em_ms(),
            iniciar_automaticamente: ciclo.iniciar_automaticamente(),
        }
    }
}

/// Modo de duracao do plano — espelho de `pomodoro_dominio::ModoDeDuracao`
/// na fronteira Rust<->TS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ModoDeDuracao {
    Global,
    Individual,
}

impl From<pomodoro_dominio::ModoDeDuracao> for ModoDeDuracao {
    fn from(modo: pomodoro_dominio::ModoDeDuracao) -> Self {
        match modo {
            pomodoro_dominio::ModoDeDuracao::Global => ModoDeDuracao::Global,
            pomodoro_dominio::ModoDeDuracao::Individual => ModoDeDuracao::Individual,
        }
    }
}

impl From<ModoDeDuracao> for pomodoro_dominio::ModoDeDuracao {
    fn from(modo: ModoDeDuracao) -> Self {
        match modo {
            ModoDeDuracao::Global => pomodoro_dominio::ModoDeDuracao::Global,
            ModoDeDuracao::Individual => pomodoro_dominio::ModoDeDuracao::Individual,
        }
    }
}

/// Atividade de pausa (PRD §7.1): nome e URL independentemente opcionais.
/// `rotulo` vem pronto do dominio — o React nao reimplementa a regra de
/// fallback (nome -> hostname sem "www." -> "atividade").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AtividadeDoPlano {
    pub nome: Option<String>,
    pub url: Option<String>,
    pub rotulo: String,
}

impl From<&pomodoro_dominio::Atividade> for AtividadeDoPlano {
    fn from(atividade: &pomodoro_dominio::Atividade) -> Self {
        Self {
            nome: atividade.nome().map(str::to_string),
            url: atividade.url().map(|url| url.como_str().to_string()),
            rotulo: atividade.rotulo(),
        }
    }
}

/// Duracoes de foco/pausa e atividade de uma sessao do plano individual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessaoDoPlano {
    #[specta(type = Number)]
    pub foco_ms: i64,
    #[specta(type = Number)]
    pub pausa_ms: i64,
    pub atividade: AtividadeDoPlano,
}

impl From<&pomodoro_dominio::Sessao> for SessaoDoPlano {
    fn from(sessao: &pomodoro_dominio::Sessao) -> Self {
        Self {
            foco_ms: sessao.foco().em_ms(),
            pausa_ms: sessao.pausa().em_ms(),
            atividade: sessao.atividade().into(),
        }
    }
}

/// Retrato do plano de duracoes: modo ativo, duracoes globais e o plano
/// individual completo — os dois conjuntos sempre presentes, coexistindo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDoPlano {
    pub modo: ModoDeDuracao,
    pub total_sessoes: u8,
    #[specta(type = Number)]
    pub duracao_global_foco_ms: i64,
    #[specta(type = Number)]
    pub duracao_global_pausa_ms: i64,
    pub atividade_global: AtividadeDoPlano,
    pub plano_individual: Vec<SessaoDoPlano>,
}

impl EstadoDoPlano {
    pub fn de(plano: &PlanoDoCiclo) -> Self {
        Self {
            modo: plano.modo().into(),
            total_sessoes: plano.total_sessoes(),
            duracao_global_foco_ms: plano.duracao_global_foco().em_ms(),
            duracao_global_pausa_ms: plano.duracao_global_pausa().em_ms(),
            atividade_global: plano.atividade_global().into(),
            plano_individual: plano.plano_individual().iter().map(Into::into).collect(),
        }
    }
}

/// Contadores do dia corrente, pro rodape da tela principal (PRD §12). Nao
/// expoe o historico de 90 dias inteiro — Estatisticas esta fora da v1, e
/// nada mais consome os dias passados hoje.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDoHistorico {
    pub sessoes_concluidas_hoje: u32,
    #[specta(type = Number)]
    pub tempo_de_foco_hoje_ms: i64,
}

impl EstadoDoHistorico {
    pub fn de(dia: ContadoresDoDia) -> Self {
        Self {
            sessoes_concluidas_hoje: dia.sessoes_concluidas(),
            tempo_de_foco_hoje_ms: dia.tempo_de_foco().em_ms(),
        }
    }
}

/// Se o modo de bloqueio (PRD §7) esta ativo agora. Emitido so na transicao
/// (entrada/saida), nunca a 1Hz — o React so precisa disto pra desativar os
/// atalhos do app (M/Esc) enquanto o bloqueio segura a tela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDoBloqueio {
    pub ativo: bool,
}

#[cfg(test)]
mod testes {
    use super::*;
    use pomodoro_dominio::{Duracao, QuantidadeDeSessoes};

    fn total(quantidade: u8) -> QuantidadeDeSessoes {
        QuantidadeDeSessoes::nova(quantidade).expect("teste")
    }

    #[test]
    fn estado_ocioso_nao_tem_prazo() {
        let ciclo = CicloEmExecucao::novo(
            total(4),
            Duracao::de_minutos(25),
            Duracao::de_minutos(5),
            true,
        );
        let estado = EstadoDaTela::de(&ciclo, Instante::desde_epoca_ms(0));
        assert_eq!(estado.prazo_epoca_ms, None);
        assert!(!estado.rodando);
        assert_eq!(estado.restante_ms, Duracao::de_minutos(25).em_ms());
        assert!(estado.iniciar_automaticamente);
    }

    #[test]
    fn estado_correndo_carrega_o_prazo_absoluto() {
        let ciclo = CicloEmExecucao::novo(
            total(4),
            Duracao::de_minutos(25),
            Duracao::de_minutos(5),
            true,
        )
        .alternar_execucao(Instante::desde_epoca_ms(1_000));
        let estado = EstadoDaTela::de(&ciclo, Instante::desde_epoca_ms(1_000));
        assert_eq!(
            estado.prazo_epoca_ms,
            Some(1_000 + Duracao::de_minutos(25).em_ms())
        );
        assert!(estado.rodando);
    }

    #[test]
    fn estado_do_historico_reflete_os_contadores_do_dia() {
        let dia = pomodoro_dominio::ContadoresDoDia::reconstruir(
            pomodoro_dominio::Data::de(2026, 8, 23),
            2,
            Duracao::de_minutos(37),
            0,
        );
        let estado = EstadoDoHistorico::de(dia);
        assert_eq!(estado.sessoes_concluidas_hoje, 2);
        assert_eq!(
            estado.tempo_de_foco_hoje_ms,
            Duracao::de_minutos(37).em_ms()
        );
    }

    #[test]
    fn estado_do_plano_reflete_modo_duracoes_globais_e_plano_individual() {
        let plano = PlanoDoCiclo::novo(total(3), Duracao::de_minutos(25), Duracao::de_minutos(5));
        let estado = EstadoDoPlano::de(&plano);
        assert_eq!(estado.modo, ModoDeDuracao::Global);
        assert_eq!(estado.total_sessoes, 3);
        assert_eq!(
            estado.duracao_global_foco_ms,
            Duracao::de_minutos(25).em_ms()
        );
        assert_eq!(estado.plano_individual.len(), 3);
        assert_eq!(
            estado.plano_individual[0].foco_ms,
            Duracao::de_minutos(25).em_ms()
        );
        assert_eq!(estado.atividade_global.rotulo, "atividade");
        assert_eq!(estado.plano_individual[0].atividade.rotulo, "atividade");
    }

    #[test]
    fn atividade_do_plano_expoe_nome_url_e_rotulo() {
        let url = pomodoro_dominio::UrlDeAtividade::nova("https://www.exemplo.com").expect("teste");
        let atividade = pomodoro_dominio::Atividade::nova(None, Some(url));
        let dto = AtividadeDoPlano::from(&atividade);
        assert_eq!(dto.nome, None);
        assert_eq!(dto.url, Some("https://www.exemplo.com/".to_string()));
        assert_eq!(dto.rotulo, "exemplo.com");
    }
}
