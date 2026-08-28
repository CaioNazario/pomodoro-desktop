use crate::Configuracao;
use pomodoro_dominio::{
    Atividade, ContadoresDoDia, Data, Duracao, HistoricoDiario, ModoDeDuracao, PlanoDoCiclo,
    Sessao, UrlDeAtividade,
};
use serde::{Deserialize, Serialize};

/// Versao do formato persistido. Bump aqui exige migracao explicita —
/// versao desconhecida nunca e lida como se fosse a atual.
pub(crate) const VERSAO_ATUAL: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConfiguracaoToml {
    pub(crate) versao: u32,
    modo: ModoToml,
    duracao_global_foco_ms: i64,
    duracao_global_pausa_ms: i64,
    #[serde(default)]
    atividade_global: AtividadeToml,
    plano_individual: Vec<SessaoToml>,
    iniciar_automaticamente: bool,
    #[serde(default)]
    historico: Vec<DiaToml>,
    #[serde(default)]
    posicao_do_widget: Option<(i32, i32)>,
}

#[derive(Debug, Serialize, Deserialize)]
enum ModoToml {
    Global,
    Individual,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AtividadeToml {
    nome: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessaoToml {
    foco_ms: i64,
    pausa_ms: i64,
    #[serde(default)]
    atividade: AtividadeToml,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiaToml {
    ano: u16,
    mes: u8,
    dia: u8,
    sessoes_concluidas: u32,
    tempo_de_foco_ms: i64,
    pausas_interrompidas: u32,
}

impl ConfiguracaoToml {
    pub(crate) fn de(config: &Configuracao) -> Self {
        Self {
            versao: VERSAO_ATUAL,
            modo: ModoToml::de(config.plano.modo()),
            duracao_global_foco_ms: config.plano.duracao_global_foco().em_ms(),
            duracao_global_pausa_ms: config.plano.duracao_global_pausa().em_ms(),
            atividade_global: AtividadeToml::de(config.plano.atividade_global()),
            plano_individual: config
                .plano
                .plano_individual()
                .iter()
                .map(SessaoToml::de)
                .collect(),
            iniciar_automaticamente: config.iniciar_automaticamente,
            historico: config.historico.dias().map(DiaToml::de).collect(),
            posicao_do_widget: config.posicao_do_widget,
        }
    }

    pub(crate) fn para_configuracao(self) -> Configuracao {
        let plano = PlanoDoCiclo::reconstruir(
            self.modo.para_dominio(),
            Duracao::de_ms(self.duracao_global_foco_ms).unwrap_or(Duracao::ZERO),
            Duracao::de_ms(self.duracao_global_pausa_ms).unwrap_or(Duracao::ZERO),
            self.atividade_global.para_dominio(),
            self.plano_individual
                .into_iter()
                .map(SessaoToml::para_dominio)
                .collect(),
        );
        let historico = HistoricoDiario::reconstruir(
            self.historico
                .into_iter()
                .map(DiaToml::para_dominio)
                .collect(),
        );
        Configuracao {
            plano,
            iniciar_automaticamente: self.iniciar_automaticamente,
            historico,
            posicao_do_widget: self.posicao_do_widget,
        }
    }
}

impl ModoToml {
    fn de(modo: ModoDeDuracao) -> Self {
        match modo {
            ModoDeDuracao::Global => Self::Global,
            ModoDeDuracao::Individual => Self::Individual,
        }
    }

    fn para_dominio(self) -> ModoDeDuracao {
        match self {
            Self::Global => ModoDeDuracao::Global,
            Self::Individual => ModoDeDuracao::Individual,
        }
    }
}

impl AtividadeToml {
    fn de(atividade: &Atividade) -> Self {
        Self {
            nome: atividade.nome().map(str::to_string),
            url: atividade.url().map(|url| url.como_str().to_string()),
        }
    }

    /// URL persistida que nao valida mais (arquivo editado a mao, ou regra
    /// de esquema mudou) e descartada silenciosamente — mesma filosofia de
    /// `Duracao::de_ms(..).unwrap_or(Duracao::ZERO)`: degradar, nunca panicar.
    fn para_dominio(self) -> Atividade {
        let url = self.url.and_then(|bruta| UrlDeAtividade::nova(&bruta).ok());
        Atividade::nova(self.nome, url)
    }
}

impl SessaoToml {
    fn de(sessao: &Sessao) -> Self {
        Self {
            foco_ms: sessao.foco().em_ms(),
            pausa_ms: sessao.pausa().em_ms(),
            atividade: AtividadeToml::de(sessao.atividade()),
        }
    }

    fn para_dominio(self) -> Sessao {
        Sessao::nova(
            Duracao::de_ms(self.foco_ms).unwrap_or(Duracao::ZERO),
            Duracao::de_ms(self.pausa_ms).unwrap_or(Duracao::ZERO),
        )
        .com_atividade(self.atividade.para_dominio())
    }
}

impl DiaToml {
    fn de(dia: ContadoresDoDia) -> Self {
        Self {
            ano: dia.dia_de_referencia().ano(),
            mes: dia.dia_de_referencia().mes(),
            dia: dia.dia_de_referencia().dia(),
            sessoes_concluidas: dia.sessoes_concluidas(),
            tempo_de_foco_ms: dia.tempo_de_foco().em_ms(),
            pausas_interrompidas: dia.pausas_interrompidas(),
        }
    }

    fn para_dominio(self) -> ContadoresDoDia {
        ContadoresDoDia::reconstruir(
            Data::de(self.ano, self.mes, self.dia),
            self.sessoes_concluidas,
            Duracao::de_ms(self.tempo_de_foco_ms).unwrap_or(Duracao::ZERO),
            self.pausas_interrompidas,
        )
    }
}
